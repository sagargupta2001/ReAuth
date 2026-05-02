use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::{Error, Result};
use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

pub struct PasskeyEnrollAuthenticator {
    passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>,
}

impl PasskeyEnrollAuthenticator {
    pub fn new(passkey_settings_repo: Arc<dyn RealmPasskeySettingsRepository>) -> Self {
        Self {
            passkey_settings_repo,
        }
    }
}

#[async_trait]
impl LifecycleNode for PasskeyEnrollAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "passkey_enroll_authenticator",
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
                output: "skip".to_string(),
            });
        }

        if session.user_id.is_none()
            && session
                .context
                .get("user_id")
                .and_then(|value| value.as_str())
                .is_none()
        {
            return Ok(NodeOutcome::Continue {
                output: "skip".to_string(),
            });
        }

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.passkey_enroll".to_string(),
            context: json!({
                "passkey_enrollment": true,
                "passkeys_enabled": settings.enabled,
                "can_skip": true,
                "auth_session_id": session.id.to_string(),
                "template_key": "passkey_enroll"
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "passkey_enroll_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        if input
            .get("action")
            .and_then(|value| value.as_str())
            .is_some_and(|action| action == "skip")
        {
            return Ok(NodeOutcome::Continue {
                output: "skip".to_string(),
            });
        }

        if input.get("passkey_credential_id").is_some() {
            if let Some(user_id) = input
                .get("passkey_user_id")
                .and_then(|value| value.as_str())
            {
                let parsed_user_id = Uuid::parse_str(user_id).map_err(Error::from)?;
                session.user_id = Some(parsed_user_id);
                session.update_context("user_id", json!(parsed_user_id.to_string()));
            }
            return Ok(NodeOutcome::Continue {
                output: "success".to_string(),
            });
        }

        Ok(NodeOutcome::Continue {
            output: "failure".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::realm_passkey_settings_repository::RealmPasskeySettingsRepository;
    use async_trait::async_trait;
    use serde_json::json;
    use uuid::Uuid;

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
    async fn execute_returns_skip_when_passkeys_disabled() {
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
        let node = PasskeyEnrollAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        let outcome = node
            .execute(&mut session)
            .await
            .expect("execute should pass");
        assert!(matches!(
            outcome,
            NodeOutcome::Continue { ref output } if output == "skip"
        ));
    }

    #[tokio::test]
    async fn execute_suspends_when_enabled_and_user_present() {
        let realm_id = Uuid::new_v4();
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
        let node = PasskeyEnrollAuthenticator::new(repo);

        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        session.user_id = Some(Uuid::new_v4());
        let outcome = node
            .execute(&mut session)
            .await
            .expect("execute should pass");
        assert!(matches!(
            outcome,
            NodeOutcome::SuspendForUI { ref screen, .. } if screen == "core.auth.passkey_enroll"
        ));
    }

    #[tokio::test]
    async fn handle_input_routes_skip_and_success() {
        let realm_id = Uuid::new_v4();
        let repo = Arc::new(StubPasskeySettingsRepo {
            settings: Some(RealmPasskeySettings::defaults(realm_id)),
        });
        let node = PasskeyEnrollAuthenticator::new(repo);
        let mut session = AuthenticationSession::new(realm_id, Uuid::new_v4(), "node".to_string());
        let user_id = Uuid::new_v4();

        let skip = node
            .handle_input(&mut session, json!({ "action": "skip" }))
            .await
            .expect("skip should pass");
        assert!(matches!(
            skip,
            NodeOutcome::Continue { ref output } if output == "skip"
        ));

        let success = node
            .handle_input(
                &mut session,
                json!({
                    "passkey_credential_id": "cred-1",
                    "passkey_user_id": user_id.to_string()
                }),
            )
            .await
            .expect("success should pass");
        assert!(matches!(
            success,
            NodeOutcome::Continue { ref output } if output == "success"
        ));
        assert_eq!(session.user_id, Some(user_id));

        let failure = node
            .handle_input(&mut session, json!({}))
            .await
            .expect("failure path should pass");
        assert!(matches!(
            failure,
            NodeOutcome::Continue { ref output } if output == "failure"
        ));
    }
}
