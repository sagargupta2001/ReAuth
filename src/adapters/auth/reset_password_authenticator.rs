use crate::application::audit_service::AuditService;
use crate::application::user_service::UserService;
use crate::domain::audit::NewAuditEvent;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::{Error, Result};
use crate::ports::auth_session_action_repository::AuthSessionActionRepository;
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use crate::ports::session_repository::SessionRepository;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const MIN_PASSWORD_LENGTH: usize = 8;

pub struct ResetPasswordAuthenticator {
    user_service: Arc<UserService>,
    session_repo: Arc<dyn SessionRepository>,
    audit_service: Arc<AuditService>,
    recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
    action_repo: Arc<dyn AuthSessionActionRepository>,
}

impl ResetPasswordAuthenticator {
    pub fn new(
        user_service: Arc<UserService>,
        session_repo: Arc<dyn SessionRepository>,
        audit_service: Arc<AuditService>,
        recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
        action_repo: Arc<dyn AuthSessionActionRepository>,
    ) -> Self {
        Self {
            user_service,
            session_repo,
            audit_service,
            recovery_settings_repo,
            action_repo,
        }
    }

    fn extract_user_id_from_payload(payload: &Value) -> Option<Uuid> {
        payload
            .get("user_id")
            .and_then(|value| value.as_str())
            .and_then(|raw| Uuid::parse_str(raw).ok())
    }

    async fn resolve_action_payload(
        &self,
        session: &mut AuthenticationSession,
    ) -> Result<Option<Value>> {
        if let Some(payload) = session.context.get("action_payload").cloned() {
            return Ok(Some(payload));
        }

        let action_id = session
            .context
            .get("action_result")
            .and_then(|value| value.get("action_id"))
            .and_then(|value| value.as_str())
            .and_then(|raw| Uuid::parse_str(raw).ok());

        let Some(action_id) = action_id else {
            return Ok(None);
        };

        let Some(action) = self.action_repo.find_by_id(&action_id).await? else {
            return Ok(None);
        };

        if action.realm_id != session.realm_id {
            return Ok(None);
        }

        if action.action_type != "reset_credentials" {
            return Ok(None);
        }

        let payload = action.payload.clone();
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("action_payload".to_string(), payload.clone());
        }
        Ok(Some(payload))
    }

    async fn resolve_user_id(
        &self,
        session: &mut AuthenticationSession,
        payload: &Value,
    ) -> Result<Option<Uuid>> {
        if let Some(user_id) = Self::extract_user_id_from_payload(payload) {
            return Ok(Some(user_id));
        }

        let identifier = payload
            .get("identifier")
            .and_then(|value| value.as_str())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let Some(identifier) = identifier else {
            return Ok(None);
        };

        let Some(user) = self
            .user_service
            .find_by_username(&session.realm_id, identifier)
            .await?
        else {
            return Ok(None);
        };

        if let Some(ctx) = session.context.as_object_mut() {
            let mut updated_payload = payload.clone();
            if let Value::Object(ref mut map) = updated_payload {
                map.insert("user_id".to_string(), json!(user.id.to_string()));
            }
            ctx.insert("action_payload".to_string(), updated_payload);
        }

        Ok(Some(user.id))
    }

    async fn reject_request(
        &self,
        session: &mut AuthenticationSession,
        reason: &str,
    ) -> Result<NodeOutcome> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("error".to_string(), json!(reason));
        } else {
            session.context = json!({ "error": reason });
        }

        Ok(NodeOutcome::Reject {
            error: reason.to_string(),
        })
    }
}

#[async_trait]
impl LifecycleNode for ResetPasswordAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "reset_password_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let previous_error = session.context.get("error").cloned();
        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.reset_password".to_string(),
            context: json!({
                "error": previous_error,
                "min_password_length": MIN_PASSWORD_LENGTH,
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "reset_password_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let password = input
            .get("password")
            .and_then(|value| value.as_str())
            .ok_or_else(|| Error::Validation("Password is required".to_string()))?;

        if password.len() < MIN_PASSWORD_LENGTH {
            return self
                .reject_request(
                    session,
                    &format!(
                        "Password must be at least {} characters",
                        MIN_PASSWORD_LENGTH
                    ),
                )
                .await;
        }

        if let Some(confirm) = input
            .get("password_confirm")
            .or_else(|| input.get("confirm_password"))
            .or_else(|| input.get("password_confirmation"))
            .and_then(|value| value.as_str())
        {
            if confirm != password {
                return self.reject_request(session, "Passwords do not match").await;
            }
        }

        let mut user_id: Option<Uuid> = None;
        if let Some(payload) = self.resolve_action_payload(session).await? {
            user_id = self.resolve_user_id(session, &payload).await?;
            if user_id.is_none() {
                return self
                    .reject_request(session, "Invalid or expired reset token")
                    .await;
            }
        }

        if user_id.is_none() {
            let force_reset_flow = session
                .context
                .get("force_password_reset")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if force_reset_flow {
                user_id = session.user_id;
            }
        }

        let Some(user_id) = user_id else {
            return self
                .reject_request(session, "Invalid or expired reset token")
                .await;
        };

        self.user_service
            .update_password(session.realm_id, user_id, password)
            .await?;

        let recovery_settings = self
            .recovery_settings_repo
            .find_by_realm_id(&session.realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(session.realm_id));

        if recovery_settings.revoke_sessions_on_reset {
            self.session_repo
                .revoke_all_for_user(&session.realm_id, &user_id)
                .await?;
        }

        if let Err(err) = self
            .audit_service
            .record(NewAuditEvent {
                realm_id: session.realm_id,
                actor_user_id: None,
                action: "recovery.password_reset".to_string(),
                target_type: "user".to_string(),
                target_id: Some(user_id.to_string()),
                metadata: json!({ "source": "reset_credentials" }),
            })
            .await
        {
            tracing::warn!("Failed to write password reset audit event: {}", err);
        }

        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("error");
        }

        Ok(NodeOutcome::Continue {
            output: "success".to_string(),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "reset_password_authenticator",
            phase = "on_exit"
        )
    )]
    async fn on_exit(&self, session: &mut AuthenticationSession) -> Result<()> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("password");
            ctx.remove("password_confirm");
            ctx.remove("confirm_password");
            ctx.remove("password_confirmation");
            ctx.remove("error");
            ctx.remove("action_payload");
            ctx.remove("force_password_reset");
        }
        Ok(())
    }
}
