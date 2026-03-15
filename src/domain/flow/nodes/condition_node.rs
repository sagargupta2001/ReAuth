use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct ConditionNodeProvider;

impl NodeProvider for ConditionNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.condition"
    }

    fn display_name(&self) -> &'static str {
        "Condition"
    }

    fn description(&self) -> &'static str {
        "Branch the flow based on a session context value."
    }

    fn icon(&self) -> &'static str {
        "Split"
    }

    fn category(&self) -> &'static str {
        "Logic"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["default"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["true", "false"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "context_path": {
                    "type": "string",
                    "title": "Context Path",
                    "description": "Dot-separated path in session context (ex: oidc.prompt or recovery.identifier)."
                },
                "operator": {
                    "type": "string",
                    "title": "Operator",
                    "default": "exists",
                    "enum": [
                        "exists",
                        "equals",
                        "not_equals",
                        "contains",
                        "starts_with",
                        "ends_with",
                        "gt",
                        "gte",
                        "lt",
                        "lte",
                        "true",
                        "false"
                    ]
                },
                "compare_value": {
                    "type": "string",
                    "title": "Compare Value",
                    "description": "Optional. If valid JSON, it is parsed before comparison."
                }
            },
            "required": ["context_path", "operator"]
        })
    }
}
