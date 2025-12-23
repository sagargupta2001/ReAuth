use crate::domain::auth_flow::LoginSession;
use crate::domain::auth_session::AuthenticationSession;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The result of a node's execution phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum NodeOutcome {
    /// Logic Node: Successfully processed, move to the next node immediately.
    /// The `output` string matches one of the edges in your graph (e.g., "success", "true", "false").
    Continue { output: String },

    /// UI Node: Stop execution and send an instruction to the Frontend.
    /// The executor will save state and wait for a POST /execute.
    SuspendForUI {
        screen: String, // e.g., "login-password"
        context: Value, // e.g., { "error": "Invalid password" }
    },

    /// Async Node: Stop execution and wait for an external event (Webhook/MagicLink).
    SuspendForAsync,

    /// Validation Failure: The user input was invalid (e.g., wrong password).
    /// Stay on the SAME node and re-render the UI with an error.
    Reject { error: String },

    /// Terminal Success: The flow is finished. Issue tokens.
    FlowSuccess { user_id: uuid::Uuid },

    /// Terminal Failure: The flow is finished. Deny roles.
    FlowFailure { reason: String },
}

/// The Lifecycle Contract.
/// Every node in your graph (Password, OTP, Script, etc.) must implement this.
#[async_trait]
pub trait LifecycleNode: Send + Sync {
    /// Phase 1: ON ENTER
    /// Runs immediately when the token lands on this node.
    /// Use this for: Rate limiting, initializing variables, generating nonces.
    async fn on_enter(&self, _session: &mut AuthenticationSession) -> Result<()> {
        Ok(())
    }

    /// Phase 2: EXECUTE
    /// Runs immediately after on_enter.
    /// Use this to: Decide if we need to show UI, or if we can proceed automatically.
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome>;

    /// Phase 3: HANDLE INPUT
    /// Runs when the user submits data (POST /execute) for this specific node.
    /// Use this to: Verify passwords, check OTP codes.
    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: Value,
    ) -> Result<NodeOutcome> {
        // Default: Logic nodes do not accept user input.
        Ok(NodeOutcome::Reject {
            error: "This node does not accept input".to_string(),
        })
    }

    /// Phase 4: ON EXIT
    /// Runs just before the token leaves this node.
    /// Use this for: Cleanup, auditing "Step Completed".
    async fn on_exit(&self, _session: &mut AuthenticationSession) -> Result<()> {
        Ok(())
    }
}
