use super::FlowCompiler;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::{ExecutionPlan, StepType};
use crate::error::Error;
use serde_json::json;

fn registry_with_basic_nodes() -> RuntimeRegistry {
    let mut registry = RuntimeRegistry::new();
    registry.register_definition("core.start", StepType::Logic);
    registry.register_definition("core.terminal.allow", StepType::Terminal);
    registry
}

fn compile(json: serde_json::Value, registry: &RuntimeRegistry) -> ExecutionPlan {
    FlowCompiler::compile(json, registry).expect("compile")
}

#[test]
fn compile_requires_nodes() {
    let registry = registry_with_basic_nodes();
    let err = FlowCompiler::compile(json!({"edges": []}), &registry).unwrap_err();
    assert!(matches!(err, Error::Validation(message) if message.contains("Missing nodes")));
}

#[test]
fn compile_requires_edges() {
    let registry = registry_with_basic_nodes();
    let err = FlowCompiler::compile(json!({"nodes": []}), &registry).unwrap_err();
    assert!(matches!(err, Error::Validation(message) if message.contains("Missing edges")));
}

#[test]
fn compile_rejects_unknown_node_types() {
    let registry = registry_with_basic_nodes();
    let err = FlowCompiler::compile(
        json!({
            "nodes": [{"id": "start", "type": "core.unknown"}],
            "edges": []
        }),
        &registry,
    )
    .unwrap_err();

    assert!(matches!(err, Error::Validation(message) if message.contains("Unknown node type")));
}

#[test]
fn compile_builds_execution_plan_with_default_handles() {
    let registry = registry_with_basic_nodes();
    let plan = compile(
        json!({
            "nodes": [
                {"id": "start", "type": "core.start", "data": {"config": {"auth_type": "core.start"}}},
                {"id": "end", "type": "core.terminal.allow"}
            ],
            "edges": [
                {"source": "start", "target": "end"}
            ]
        }),
        &registry,
    );

    assert_eq!(plan.start_node_id, "start");
    let start = plan.nodes.get("start").expect("start node");
    assert_eq!(start.step_type, StepType::Logic);
    assert_eq!(start.next.get("default").unwrap(), "end");
    assert_eq!(start.config.get("auth_type").unwrap(), "core.start");
}

#[test]
fn compile_uses_source_handles_when_present() {
    let registry = registry_with_basic_nodes();
    let plan = compile(
        json!({
            "nodes": [
                {"id": "start", "type": "core.start"},
                {"id": "end", "type": "core.terminal.allow"}
            ],
            "edges": [
                {"source": "start", "target": "end", "sourceHandle": "success"}
            ]
        }),
        &registry,
    );

    let start = plan.nodes.get("start").expect("start node");
    assert_eq!(start.next.get("success").unwrap(), "end");
}
