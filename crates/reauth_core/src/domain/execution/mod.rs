pub mod lifecycle;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// The final artifact stored in 'flow_versions'
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub start_node_id: String,
    pub nodes: HashMap<String, ExecutionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    pub id: String,
    pub step_type: StepType,
    // Where to go next?
    // Key = Output Handle (e.g., "success", "failure", "true", "false")
    // Value = Target Node ID
    pub next: HashMap<String, String>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Authenticator, // Renders a UI (LoginForm, etc.)
    Logic,         // Internal calculation (If/Else)
    Terminal,      // Ends the flow (Issue Token, Deny)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ExecutionResult {
    /// Stop and show a screen (e.g., Login Form)
    Challenge {
        screen_id: String,
        context: serde_json::Value,
    },
    /// Flow finished successfully
    Success { redirect_url: String },
    /// Flow finished with failure
    Failure { reason: String },
}
