use super::{ExecutionNode, ExecutionPlan, ExecutionResult, StepType};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn step_type_serializes_snake_case() {
    let json = serde_json::to_string(&StepType::Authenticator).expect("serialize");
    assert_eq!(json, "\"authenticator\"");
}

#[test]
fn execution_node_defaults_missing_fields() {
    let json = r#"{"id":"node-1","step_type":"logic"}"#;
    let node: ExecutionNode = serde_json::from_str(json).expect("deserialize");

    assert!(node.next.is_empty());
    assert!(node.config.is_null());
}

#[test]
fn execution_plan_round_trip() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "start".to_string(),
        ExecutionNode {
            id: "start".to_string(),
            step_type: StepType::Logic,
            next: HashMap::new(),
            config: json!({"key": "value"}),
        },
    );

    let plan = ExecutionPlan {
        start_node_id: "start".to_string(),
        nodes,
    };

    let json = serde_json::to_string(&plan).expect("serialize");
    let decoded: ExecutionPlan = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.start_node_id, "start");
    assert!(decoded.nodes.contains_key("start"));
}

#[test]
fn execution_result_serializes_with_tagged_payload() {
    let value = serde_json::to_value(ExecutionResult::Success {
        redirect_url: "https://example.com".to_string(),
    })
    .expect("serialize");

    assert_eq!(value.get("type").unwrap(), "Success");
    assert_eq!(
        value.get("payload").unwrap().get("redirect_url").unwrap(),
        "https://example.com"
    );

    let challenge = serde_json::to_value(ExecutionResult::Challenge {
        screen_id: "login".to_string(),
        context: json!({"foo": "bar"}),
    })
    .expect("serialize");

    assert_eq!(challenge.get("type").unwrap(), "Challenge");
    assert_eq!(
        challenge.get("payload").unwrap().get("screen_id").unwrap(),
        "login"
    );
}
