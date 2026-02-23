use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use crate::ports::session_repository::SessionRepository; // For RefreshTokens

pub struct CookieAuthenticator {
    // We need the repo that stores RefreshTokens (SessionRepository)
    session_repo: Arc<dyn SessionRepository>,
}

impl CookieAuthenticator {
    pub fn new(session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { session_repo }
    }
}

#[async_trait]
impl LifecycleNode for CookieAuthenticator {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "cookie_authenticator", phase = "on_enter")
    )]
    async fn on_enter(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // No setup needed
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "cookie_authenticator", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        // 1. Extract Token ID from Context
        let token_id_str = match session.context.get("sso_token_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return Ok(NodeOutcome::Continue {
                    output: "continue".to_string(),
                })
            }
        };

        let token_id = match uuid::Uuid::parse_str(token_id_str) {
            Ok(id) => id,
            Err(_) => {
                return Ok(NodeOutcome::Continue {
                    output: "continue".to_string(),
                })
            }
        };

        // 2. Lookup Token in DB
        match self.session_repo.find_by_id(&token_id).await {
            Ok(Some(token)) => {
                // [SECURITY FIX] Realm Isolation Check
                // We MUST ensure the token belongs to the same realm as the current login attempt.
                if token.realm_id != session.realm_id {
                    tracing::warn!(
                        "CookieAuth: Cross-Realm Attack Blocked. Token Realm {} != Session Realm {}",
                        token.realm_id,
                        session.realm_id
                    );
                    // Treat as invalid -> Force user to login again for this specific realm
                    return Ok(NodeOutcome::Continue {
                        output: "continue".to_string(),
                    });
                }

                // 3. Success
                tracing::info!(
                    "CookieAuth: Valid SSO session found for user {}",
                    token.user_id
                );
                session.user_id = Some(token.user_id);
                Ok(NodeOutcome::FlowSuccess {
                    user_id: token.user_id,
                })
            }
            _ => Ok(NodeOutcome::Continue {
                output: "continue".to_string(),
            }),
        }
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "cookie_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: Value,
    ) -> Result<NodeOutcome> {
        // This node is non-interactive. It should never receive UI input.
        // If it does, we treat it as a reject/retry (though executor shouldn't send it).
        Err(crate::error::Error::System(
            "CookieAuthenticator received input".into(),
        ))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "cookie_authenticator", phase = "on_exit")
    )]
    async fn on_exit(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // Cleanup if needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::RefreshToken;
    use crate::ports::session_repository::SessionRepository;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use mockall::mock;
    use uuid::Uuid;

    mock! {
        pub SessionRepo {}
        #[async_trait]
        impl SessionRepository for SessionRepo {
            async fn save(&self, token: &RefreshToken) -> Result<()>;
            async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>>;
            async fn delete_by_id(&self, id: &Uuid) -> Result<()>;
            async fn list(&self, realm_id: &Uuid, req: &crate::domain::pagination::PageRequest) -> Result<crate::domain::pagination::PageResponse<RefreshToken>>;
        }
    }

    #[tokio::test]
    async fn execute_continues_when_no_token_in_context() {
        let mut repo = MockSessionRepo::new();
        repo.expect_find_by_id().never();

        let auth = CookieAuthenticator::new(Arc::new(repo));
        let mut session =
            AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".into());

        let result = auth.execute(&mut session).await.unwrap();
        assert!(matches!(result, NodeOutcome::Continue { .. }));
    }

    #[tokio::test]
    async fn execute_continues_when_token_not_found() {
        let token_id = Uuid::new_v4();
        let mut repo = MockSessionRepo::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(token_id))
            .returning(|_| Ok(None));

        let auth = CookieAuthenticator::new(Arc::new(repo));
        let mut session =
            AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".into());
        session.update_context("sso_token_id", token_id.to_string().into());

        let result = auth.execute(&mut session).await.unwrap();
        assert!(matches!(result, NodeOutcome::Continue { .. }));
    }

    #[tokio::test]
    async fn execute_blocks_cross_realm_attack() {
        let token_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let realm_a = Uuid::new_v4();
        let realm_b = Uuid::new_v4();

        let token = RefreshToken {
            id: token_id,
            user_id,
            realm_id: realm_a, // Token belongs to Realm A
            client_id: None,
            expires_at: Utc::now() + Duration::hours(1),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

        let mut repo = MockSessionRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(token.clone())));

        let auth = CookieAuthenticator::new(Arc::new(repo));
        // Session belongs to Realm B
        let mut session = AuthenticationSession::new(realm_b, Uuid::new_v4(), "start".into());
        session.update_context("sso_token_id", token_id.to_string().into());

        let result = auth.execute(&mut session).await.unwrap();
        // Should NOT log in, but continue to next node (likely Password)
        assert!(matches!(result, NodeOutcome::Continue { .. }));
        assert!(session.user_id.is_none());
    }

    #[tokio::test]
    async fn execute_succeeds_with_valid_token() {
        let token_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        let token = RefreshToken {
            id: token_id,
            user_id,
            realm_id,
            client_id: None,
            expires_at: Utc::now() + Duration::hours(1),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

        let mut repo = MockSessionRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(token.clone())));

        let auth = CookieAuthenticator::new(Arc::new(repo));
        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "start".into());
        session.update_context("sso_token_id", token_id.to_string().into());

        let result = auth.execute(&mut session).await.unwrap();

        if let NodeOutcome::FlowSuccess {
            user_id: authenticated_user,
        } = result
        {
            assert_eq!(authenticated_user, user_id);
        } else {
            panic!("Expected FlowSuccess, got {:?}", result);
        }
        assert_eq!(session.user_id, Some(user_id));
    }
}
