use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use tracing::instrument;

pub struct InvitationTokenNode;

#[async_trait]
impl LifecycleNode for InvitationTokenNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "invitation_token", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let invitation_id = session
            .context
            .get("invitation_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let invitation_email = session
            .context
            .get("invitation_email")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let invitation_token_hash = session
            .context
            .get("invitation_token_hash")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if invitation_id.is_none() || invitation_email.is_none() {
            return Ok(NodeOutcome::FlowFailure {
                reason: "Invitation context missing".to_string(),
            });
        }

        if invitation_token_hash.is_none() {
            // For issue-time execution this may not be set yet; it is required only on accept flows.
            if let Some(ctx) = session.context.as_object_mut() {
                ctx.insert(
                    "invitation_token_hash".to_string(),
                    serde_json::json!("pending"),
                );
            }
        }

        Ok(NodeOutcome::Continue {
            output: "valid".to_string(),
        })
    }
}
