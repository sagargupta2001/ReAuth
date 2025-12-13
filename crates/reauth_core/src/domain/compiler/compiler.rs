use super::validator::{GraphEdge, GraphNode, GraphValidator};
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::{ExecutionNode, ExecutionPlan, StepType};
use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};

pub struct FlowCompiler;

impl FlowCompiler {
    // 1. Signature Change: Accept &RuntimeRegistry
    pub fn compile(json: serde_json::Value, registry: &RuntimeRegistry) -> Result<ExecutionPlan> {
        // --- STEP 1: Parse Raw JSON ---
        let nodes_val = json
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or(Error::Validation("Missing nodes".into()))?;
        let edges_val = json
            .get("edges")
            .and_then(|v| v.as_array())
            .ok_or(Error::Validation("Missing edges".into()))?;

        let mut nodes = Vec::new();
        // Helper to map ID -> Config (JSON) for later
        let mut node_configs: HashMap<String, serde_json::Value> = HashMap::new();

        for n in nodes_val {
            let id = n["id"].as_str().unwrap().to_string();
            let type_ = n["type"].as_str().unwrap_or("default").to_string();

            // Extract config from ReactFlow format: node.data.config
            let config = n
                .get("data")
                .and_then(|d| d.get("config"))
                .cloned()
                .unwrap_or(serde_json::json!({}));

            node_configs.insert(id.clone(), config);

            nodes.push(GraphNode { id, type_ });
        }

        let mut edges = Vec::new();
        for e in edges_val {
            edges.push(GraphEdge {
                source: e["source"].as_str().unwrap().to_string(),
                target: e["target"].as_str().unwrap().to_string(),
                source_handle: e["sourceHandle"].as_str().map(|s| s.to_string()),
            });
        }

        // --- STEP 2: Validate using Registry ---
        // This fixes the "Dead end detected" error by letting the validator check StepType
        GraphValidator::validate(&nodes, &edges, registry)?;

        // --- STEP 3: Compile to Execution Plan ---
        let mut execution_nodes = HashMap::new();
        let mut start_node_id = String::new();

        // Build Adjacency Map: SourceID -> { Handle -> TargetID }
        let mut adjacency: HashMap<String, HashMap<String, String>> = HashMap::new();
        for edge in &edges {
            let handle = edge
                .source_handle
                .clone()
                .unwrap_or_else(|| "default".to_string());
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .insert(handle, edge.target.clone());
        }

        // Find Start Node (node with no incoming edges)
        let targets: HashSet<String> = edges.iter().map(|e| e.target.clone()).collect();

        for node in nodes {
            // [CRITICAL FIX] Dynamic Step Type Lookup
            // Instead of hardcoding strings, we ask the registry.
            // This ensures "core.terminal.allow" is correctly mapped to StepType::Terminal
            let step_type = registry.get_node_type(&node.type_).ok_or_else(|| {
                Error::Validation(format!(
                    "Unknown node type during compilation: {}",
                    node.type_
                ))
            })?;

            // Capture start node
            if !targets.contains(&node.id) {
                start_node_id = node.id.clone();
            }

            let next_map = adjacency.remove(&node.id).unwrap_or_default();
            let config = node_configs
                .remove(&node.id)
                .unwrap_or(serde_json::json!({}));

            execution_nodes.insert(
                node.id.clone(),
                ExecutionNode {
                    id: node.id,
                    step_type, // Uses the correct enum from registry
                    next: next_map,
                    config, // Uses the extracted config
                },
            );
        }

        if start_node_id.is_empty() && !execution_nodes.is_empty() {
            return Err(Error::Validation("No start node detected".into()));
        }

        Ok(ExecutionPlan {
            start_node_id,
            nodes: execution_nodes,
        })
    }
}
