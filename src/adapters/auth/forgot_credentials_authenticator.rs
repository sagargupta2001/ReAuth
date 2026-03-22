use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::domain::recovery_attempt::RecoveryAttempt;
use crate::error::Result;
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use crate::ports::recovery_attempt_repository::RecoveryAttemptRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::instrument;

pub struct ForgotCredentialsAuthenticator {
    recovery_attempt_repo: Arc<dyn RecoveryAttemptRepository>,
    recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
}

impl ForgotCredentialsAuthenticator {
    pub fn new(
        recovery_attempt_repo: Arc<dyn RecoveryAttemptRepository>,
        recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
    ) -> Self {
        Self {
            recovery_attempt_repo,
            recovery_settings_repo,
        }
    }

    async fn reject_request(
        &self,
        session: &mut AuthenticationSession,
        identifier: &str,
        reason: &str,
    ) -> Result<NodeOutcome> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("error".to_string(), json!(reason));
            ctx.insert("email".to_string(), json!(identifier));
        } else {
            session.context = json!({
                "error": reason,
                "email": identifier,
            });
        }

        Ok(NodeOutcome::Reject {
            error: reason.to_string(),
        })
    }

    async fn enforce_rate_limit(
        &self,
        session: &mut AuthenticationSession,
        identifier: &str,
        rate_limit_max: i64,
        rate_limit_window_minutes: i64,
    ) -> Result<Option<NodeOutcome>> {
        if rate_limit_max <= 0 {
            return Ok(None);
        }
        let now = Utc::now();
        let window = Duration::minutes(rate_limit_window_minutes.max(1));
        let mut attempt = self
            .recovery_attempt_repo
            .find(&session.realm_id, identifier)
            .await?
            .unwrap_or(RecoveryAttempt {
                realm_id: session.realm_id,
                identifier: identifier.to_string(),
                window_started_at: now,
                attempt_count: 0,
                updated_at: now,
            });

        if now.signed_duration_since(attempt.window_started_at) >= window {
            attempt.window_started_at = now;
            attempt.attempt_count = 0;
        }

        if attempt.attempt_count >= rate_limit_max {
            let retry_at = attempt.window_started_at + window;
            let reason = format!(
                "Too many recovery attempts. Try again after {}",
                retry_at.to_rfc3339()
            );
            if let Some(ctx) = session.context.as_object_mut() {
                ctx.insert("retry_at".to_string(), json!(retry_at));
                ctx.insert("email".to_string(), json!(identifier));
            } else {
                session.context = json!({
                    "retry_at": retry_at,
                    "email": identifier,
                });
            }
            return Ok(Some(NodeOutcome::Reject {
                error: reason.to_string(),
            }));
        }

        attempt.attempt_count += 1;
        attempt.updated_at = now;
        self.recovery_attempt_repo.upsert(&attempt).await?;

        Ok(None)
    }
}

#[async_trait]
impl LifecycleNode for ForgotCredentialsAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "forgot_credentials_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let previous_error = session.context.get("error").cloned();
        let email_prefill = session
            .context
            .get("email")
            .cloned()
            .or_else(|| session.context.get("username").cloned());

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.forgot_credentials".to_string(),
            context: json!({
                "email": email_prefill,
                "error": previous_error,
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "forgot_credentials_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let identifier = input
            .get("email")
            .or_else(|| input.get("username"))
            .and_then(|value| value.as_str());
        let Some(identifier) = identifier else {
            return self
                .reject_request(session, "", "Email or username is required")
                .await;
        };
        let identifier = identifier.trim();
        if identifier.is_empty() {
            return self
                .reject_request(session, identifier, "Email or username is required")
                .await;
        }

        let recovery_settings = self
            .recovery_settings_repo
            .find_by_realm_id(&session.realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(session.realm_id));

        let rate_limit_max = recovery_settings.rate_limit_max;
        let rate_limit_window_minutes = recovery_settings.rate_limit_window_minutes;

        if let Some(outcome) = self
            .enforce_rate_limit(
                session,
                identifier,
                rate_limit_max,
                rate_limit_window_minutes,
            )
            .await?
        {
            return Ok(outcome);
        }
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("retry_at");
            ctx.remove("error");
            ctx.insert("email".to_string(), json!(identifier));
            ctx.insert("username".to_string(), json!(identifier));
            ctx.insert("recovery_identifier".to_string(), json!(identifier));
        } else {
            session.context = json!({
                "email": identifier,
                "username": identifier,
                "recovery_identifier": identifier,
            });
        }

        Ok(NodeOutcome::Continue {
            output: "success".to_string(),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "forgot_credentials_authenticator",
            phase = "on_exit"
        )
    )]
    async fn on_exit(&self, session: &mut AuthenticationSession) -> Result<()> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("error");
        }
        Ok(())
    }
}
