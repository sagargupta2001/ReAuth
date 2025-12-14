use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct ScriptNode;

impl NodeProvider for ScriptNode {
    fn id(&self) -> &'static str {
        "core.logic.script"
    }
    fn display_name(&self) -> &'static str {
        "Execution Script"
    }
    fn description(&self) -> &'static str {
        "Run custom internal logic."
    }
    fn icon(&self) -> &'static str {
        "Code"
    }
    fn category(&self) -> &'static str {
        "Logic"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["next", "error"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "script_name": { "type": "string", "title": "Script Name" }
            }
        })
    }
}
