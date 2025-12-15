use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

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
    async fn on_enter(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // No setup needed
        Ok(())
    }

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

    async fn on_exit(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // Cleanup if needed
        Ok(())
    }
}
