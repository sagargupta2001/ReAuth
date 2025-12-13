use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct ConditionNode;

impl NodeProvider for ConditionNode {
    fn id(&self) -> &'static str {
        "core.logic.condition"
    }
    fn display_name(&self) -> &'static str {
        "Condition Check"
    }
    fn description(&self) -> &'static str {
        "Branch flow based on user or session data."
    }
    fn icon(&self) -> &'static str {
        "Split"
    }
    fn category(&self) -> &'static str {
        "Logic"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["true", "false"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["variable", "operator", "value"],
            "properties": {
                "variable": {
                    "type": "string",
                    "description": "e.g. user.email_verified, context.ip_address",
                    "title": "Variable"
                },
                "operator": {
                    "type": "string",
                    "enum": ["equals", "not_equals", "contains", "starts_with"],
                    "default": "equals",
                    "title": "Operator"
                },
                "value": { "type": "string", "title": "Comparison Value" }
            }
        })
    }
}
