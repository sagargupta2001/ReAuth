use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::instrument;

pub struct VerifyEmailOtpAuthenticator;

impl VerifyEmailOtpAuthenticator {
    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn auto_continue(session: &AuthenticationSession) -> bool {
        Self::node_config(session)
            .get("auto_continue")
            .and_then(|value| value.as_bool())
            .unwrap_or(true)
    }

    fn action_is_verified(session: &AuthenticationSession) -> bool {
        session
            .context
            .get("action_result")
            .and_then(|value| value.get("action_type"))
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == "email_verify")
    }
}

#[async_trait]
impl LifecycleNode for VerifyEmailOtpAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "verify_email_otp_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        if Self::action_is_verified(session) {
            session.update_context("email_verified", json!(true));
            if Self::auto_continue(session) {
                return Ok(NodeOutcome::Continue {
                    output: "success".to_string(),
                });
            }
        }

        let previous_error = session.context.get("error").cloned();

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.verify_email_otp".to_string(),
            context: json!({
                "error": previous_error,
                "verified": Self::action_is_verified(session),
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "verify_email_otp_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        _input: Value,
    ) -> Result<NodeOutcome> {
        if Self::action_is_verified(session) {
            return Ok(NodeOutcome::Continue {
                output: "success".to_string(),
            });
        }

        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert(
                "error".to_string(),
                json!("Verification token is missing or expired."),
            );
        }

        Ok(NodeOutcome::Reject {
            error: "Verification token is missing or expired.".to_string(),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "verify_email_otp_authenticator",
            phase = "on_exit"
        )
    )]
    async fn on_exit(&self, session: &mut AuthenticationSession) -> Result<()> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("error");
            ctx.remove("action_result");
            ctx.remove("action_payload");
        }
        Ok(())
    }
}
