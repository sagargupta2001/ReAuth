use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::passkey_runtime::{PASSKEY_REAUTH_AT_KEY, PASSKEY_REAUTH_USER_ID_KEY};
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub struct PasskeyAssertAuthenticator {
    passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
}

impl PasskeyAssertAuthenticator {
    pub fn new(passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>) -> Self {
        Self {
            passkey_settings_repo,
        }
    }

    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn node_intent(session: &AuthenticationSession) -> String {
        Self::node_config(session)
            .get("intent")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("login")
            .to_lowercase()
    }

    fn should_require_reauth(session: &AuthenticationSession) -> bool {
        Self::node_intent(session) == "reauth"
    }

    fn has_fresh_reauth(session: &AuthenticationSession, max_age_secs: i64) -> bool {
        if max_age_secs <= 0 {
            return false;
        }

        let last_reauth_at = session
            .context
            .get(PASSKEY_REAUTH_AT_KEY)
            .and_then(|value| value.as_str())
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let Some(last_reauth_at) = last_reauth_at else {
            return false;
        };

        if Utc::now() - last_reauth_at > Duration::seconds(max_age_secs) {
            return false;
        }

        let expected_user = session.user_id.map(|value| value.to_string()).or_else(|| {
            session
                .context
                .get("user_id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        });
        let Some(expected_user) = expected_user else {
            return true;
        };

        session
            .context
            .get(PASSKEY_REAUTH_USER_ID_KEY)
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == expected_user)
    }
}

#[async_trait]
impl LifecycleNode for PasskeyAssertAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "passkey_assert_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let settings = self
            .passkey_settings_repo
            .find_by_realm_id(&session.realm_id)
            .await?
            .unwrap_or_else(|| RealmPasskeySettings::defaults(session.realm_id));

        if !settings.enabled {
            return Ok(NodeOutcome::Continue {
                output: "fallback".to_string(),
            });
        }

        let intent = Self::node_intent(session);
        if Self::should_require_reauth(session)
            && Self::has_fresh_reauth(session, settings.reauth_max_age_secs)
        {
            return Ok(NodeOutcome::Continue {
                output: "success".to_string(),
            });
        }

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.passkey_assert".to_string(),
            context: json!({
                "fallback_allowed": settings.allow_password_fallback,
                "passkeys_enabled": settings.enabled,
                "passkey_intent": intent,
                "auth_session_id": session.id.to_string(),
                "template_key": "passkey_assert"
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "passkey_assert_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        if let Some(action) = input.get("action").and_then(|value| value.as_str()) {
            if action == "fallback" {
                return Ok(NodeOutcome::Continue {
                    output: "fallback".to_string(),
                });
            }
        }

        let user_id = input
            .get("passkey_user_id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                Error::Validation("passkey_user_id is required for passkey assertion".to_string())
            })
            .and_then(|value| Uuid::parse_str(value).map_err(Error::from))?;

        session.user_id = Some(user_id);
        session.update_context("user_id", json!(user_id.to_string()));

        if let Some(credential_id) = input
            .get("passkey_credential_id")
            .and_then(|value| value.as_str())
        {
            session.update_context("passkey_credential_id", json!(credential_id));
        }

        Ok(NodeOutcome::Continue {
            output: "success".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_session::AuthenticationSession;
    use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
    use async_trait::async_trait;
    use chrono::Utc;
    use serde_json::json;

    struct StubPasskeySettingsRepo {
        settings: Option<RealmPasskeySettings>,
    }

    #[async_trait]
    impl RealmPasskeySettingsRepository for StubPasskeySettingsRepo {
        async fn find_by_realm_id(&self, _realm_id: &Uuid) -> Result<Option<RealmPasskeySettings>> {
            Ok(self.settings.clone())
        }

        async fn upsert(&self, _settings: &RealmPasskeySettings) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn execute_returns_fallback_when_passkeys_disabled() {
        let realm_id = Uuid::new_v4();
        let repo = Arc::new(StubPasskeySettingsRepo {
            settings: Some(RealmPasskeySettings {
                realm_id,
                enabled: false,
                allow_password_fallback: true,
                discoverable_preferred: true,
                challenge_ttl_secs: 120,
                reauth_max_age_secs: 300,
            }),
        });
        let node = PasskeyAssertAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        let outcome = node
            .execute(&mut session)
            .await
            .expect("execute should pass");
        assert!(matches!(
            outcome,
            NodeOutcome::Continue { ref output } if output == "fallback"
        ));
    }

    #[tokio::test]
    async fn handle_input_sets_user_and_returns_success() {
        let realm_id = Uuid::new_v4();
        let repo = Arc::new(StubPasskeySettingsRepo {
            settings: Some(RealmPasskeySettings::defaults(realm_id)),
        });
        let node = PasskeyAssertAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        let user_id = Uuid::new_v4();
        let outcome = node
            .handle_input(
                &mut session,
                json!({
                    "passkey_user_id": user_id.to_string(),
                    "passkey_credential_id": "cred-123"
                }),
            )
            .await
            .expect("handle_input should pass");

        assert!(matches!(
            outcome,
            NodeOutcome::Continue { ref output } if output == "success"
        ));
        assert_eq!(session.user_id, Some(user_id));
        assert_eq!(
            session
                .context
                .get("passkey_credential_id")
                .and_then(|value| value.as_str()),
            Some("cred-123")
        );
    }

    #[tokio::test]
    async fn handle_input_allows_explicit_fallback() {
        let realm_id = Uuid::new_v4();
        let repo = Arc::new(StubPasskeySettingsRepo {
            settings: Some(RealmPasskeySettings::defaults(realm_id)),
        });
        let node = PasskeyAssertAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        let outcome = node
            .handle_input(&mut session, json!({ "action": "fallback" }))
            .await
            .expect("handle_input should pass");
        assert!(matches!(
            outcome,
            NodeOutcome::Continue { ref output } if output == "fallback"
        ));
    }

    #[tokio::test]
    async fn execute_skips_reauth_when_fresh_timestamp_exists() {
        let realm_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let repo = Arc::new(StubPasskeySettingsRepo {
            settings: Some(RealmPasskeySettings {
                realm_id,
                enabled: true,
                allow_password_fallback: true,
                discoverable_preferred: true,
                challenge_ttl_secs: 120,
                reauth_max_age_secs: 300,
            }),
        });
        let node = PasskeyAssertAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        session.user_id = Some(user_id);
        session.update_context("node_config", json!({ "intent": "reauth" }));
        session.update_context(PASSKEY_REAUTH_AT_KEY, json!(Utc::now().to_rfc3339()));
        session.update_context(PASSKEY_REAUTH_USER_ID_KEY, json!(user_id.to_string()));

        let outcome = node
            .execute(&mut session)
            .await
            .expect("execute should pass");
        assert!(matches!(
            outcome,
            NodeOutcome::Continue { ref output } if output == "success"
        ));
    }
}
