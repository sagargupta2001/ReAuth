use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::instrument;

pub struct InvitationUnavailableAuthenticator;

impl InvitationUnavailableAuthenticator {
    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn config_string(config: &Value, key: &str) -> Option<String> {
        config
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn status(session: &AuthenticationSession) -> String {
        session
            .context
            .get("invitation_token_status")
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "invalid".to_string())
    }

    fn message(config: &Value, status: &str) -> String {
        match status {
            "expired" => Self::config_string(config, "expired_message").unwrap_or_else(|| {
                "This invitation link has expired. Ask your administrator to send a new invitation."
                    .to_string()
            }),
            "consumed" => Self::config_string(config, "consumed_message")
                .unwrap_or_else(|| "This invitation link has already been used.".to_string()),
            _ => Self::config_string(config, "invalid_message")
                .unwrap_or_else(|| "This invitation link is invalid.".to_string()),
        }
    }
}

#[async_trait]
impl LifecycleNode for InvitationUnavailableAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "invitation_unavailable_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let config = Self::node_config(session);
        let status = Self::status(session);
        let title = Self::config_string(&config, "title")
            .unwrap_or_else(|| "Invitation Link Unavailable".to_string());
        let template_key = Self::config_string(&config, "template_key")
            .unwrap_or_else(|| "invitation_unavailable".to_string());
        let message = Self::message(&config, &status);

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.invitation_unavailable".to_string(),
            context: json!({
                "template_key": template_key,
                "title": title,
                "message": message,
                "error": message,
                "invitation_token_status": status,
                "invitation_unavailable": true
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "invitation_unavailable_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: Value,
    ) -> Result<NodeOutcome> {
        Ok(NodeOutcome::Continue {
            output: "failure".to_string(),
        })
    }
}
