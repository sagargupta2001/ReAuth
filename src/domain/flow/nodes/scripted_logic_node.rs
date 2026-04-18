use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct ScriptedLogicNodeProvider;

impl NodeProvider for ScriptedLogicNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.scripted"
    }

    fn display_name(&self) -> &'static str {
        "Scripted Logic"
    }

    fn description(&self) -> &'static str {
        "Run server-side JavaScript to branch the flow and update session context."
    }

    fn icon(&self) -> &'static str {
        "Code2"
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
                "script": {
                    "type": "string",
                    "title": "Logic Script",
                    "description": "Handler body. Receives (input, context, signal). Return { output?: 'success' | 'failure', context?: object, remove_keys?: string[] }.",
                    "format": "code"
                }
            },
            "required": ["script"],
            "additionalProperties": false
        })
    }
}
