use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct SubflowNodeProvider;

impl NodeProvider for SubflowNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.subflow"
    }

    fn display_name(&self) -> &'static str {
        "Call Subflow"
    }

    fn description(&self) -> &'static str {
        "Enter another deployed child flow and return to this flow on success or failure."
    }

    fn icon(&self) -> &'static str {
        "Workflow"
    }

    fn category(&self) -> &'static str {
        "Logic"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["default"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "flow_type": {
                    "type": "string",
                    "title": "Subflow Type",
                    "description": "Flow deployment type to invoke, for example 'registration' or 'custom_step_up'."
                }
            },
            "required": ["flow_type"],
            "additionalProperties": false
        })
    }
}
