use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct StartNode;

impl NodeProvider for StartNode {
    fn id(&self) -> &'static str {
        "core.start"
    }
    fn display_name(&self) -> &'static str {
        "Start Flow"
    }
    fn description(&self) -> &'static str {
        "The entry point of the authentication flow."
    }
    fn icon(&self) -> &'static str {
        "Play"
    }
    fn category(&self) -> &'static str {
        "Start"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec![] // Validator checks: No inputs = Start
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["next"]
    }

    fn config_schema(&self) -> Value {
        json!({})
    }
}
