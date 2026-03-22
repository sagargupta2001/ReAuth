use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct RecoveryIssueNodeProvider;

impl NodeProvider for RecoveryIssueNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.recovery_issue"
    }

    fn display_name(&self) -> &'static str {
        "Issue Recovery Token"
    }

    fn description(&self) -> &'static str {
        "Generate a recovery token and suspend the flow for async resume."
    }

    fn icon(&self) -> &'static str {
        "ShieldAlert"
    }

    fn category(&self) -> &'static str {
        "Logic"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["default"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["issued"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("awaiting_action")
    }
}
