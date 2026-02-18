use super::NodeProvider;
use serde_json::json;

struct DummyNode;

impl NodeProvider for DummyNode {
    fn id(&self) -> &'static str {
        "dummy.node"
    }

    fn display_name(&self) -> &'static str {
        "Dummy"
    }

    fn description(&self) -> &'static str {
        "Dummy node"
    }

    fn icon(&self) -> &'static str {
        "DummyIcon"
    }

    fn category(&self) -> &'static str {
        "Logic"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["out"]
    }

    fn config_schema(&self) -> serde_json::Value {
        json!({"type": "object"})
    }
}

#[test]
fn provider_defaults_are_applied() {
    let node = DummyNode;

    assert_eq!(node.inputs(), vec!["in"]);
    assert_eq!(node.default_config(), json!({}));
}
