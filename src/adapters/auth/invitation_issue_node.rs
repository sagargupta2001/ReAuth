use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString};
use serde_json::{json, Value};
use tracing::instrument;

const INVITATION_TOKEN_LENGTH: usize = 48;

pub struct InvitationIssueNode;

impl InvitationIssueNode {
    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn resolve_string(config: &Value, key: &str) -> Option<String> {
        config
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn extract_email(session: &AuthenticationSession) -> Option<String> {
        session
            .context
            .get("invitation_email")
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn extract_invitation_id(session: &AuthenticationSession) -> Option<String> {
        session
            .context
            .get("invitation_id")
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn extract_expires_at(session: &AuthenticationSession) -> Option<DateTime<Utc>> {
        session
            .context
            .get("invitation_expires_at")
            .and_then(|value| value.as_str())
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc))
    }
}

#[async_trait]
impl LifecycleNode for InvitationIssueNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "invitation_issue_node", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let Some(email) = Self::extract_email(session) else {
            return Ok(NodeOutcome::FlowFailure {
                reason: "Invitation email missing".to_string(),
            });
        };
        let Some(invitation_id) = Self::extract_invitation_id(session) else {
            return Ok(NodeOutcome::FlowFailure {
                reason: "Invitation id missing".to_string(),
            });
        };

        let config = Self::node_config(session);
        let expires_at = Self::extract_expires_at(session).unwrap_or_else(Utc::now);
        let token = Alphanumeric.sample_string(&mut rand::rng(), INVITATION_TOKEN_LENGTH);
        let resume_path = Self::resolve_string(&config, "resume_path")
            .unwrap_or_else(|| "/invite/accept".to_string());
        let resend_path =
            Self::resolve_string(&config, "resend_path").unwrap_or_else(|| resume_path.clone());
        let resume_node_id =
            Self::resolve_string(&config, "resume_node_id").or(Some("auth-register".to_string()));

        Ok(NodeOutcome::SuspendForAsync {
            action_type: "invitation_accept".to_string(),
            token: token.clone(),
            expires_at,
            resume_node_id,
            payload: json!({
                "identifier": email,
                "invitation_id": invitation_id,
                "resume_path": resume_path,
                "resend_path": resend_path,
            }),
            screen: "core.awaiting-action".to_string(),
            context: json!({
                "message": "Invitation email is being sent.",
                "resume_token": token,
                "expires_at": expires_at,
                "action_type": "invitation_accept",
                "resume_path": resume_path,
                "resend_path": resend_path,
                "email": email,
            }),
        })
    }
}
