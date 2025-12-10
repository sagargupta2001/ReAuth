// src/domain/compiler/compiler.rs

use super::validator::{GraphEdge, GraphNode, GraphValidator};
use crate::domain::execution::{ExecutionNode, ExecutionPlan, StepType};
use crate::error::Result;
use std::collections::HashMap;

pub struct FlowCompiler;

impl FlowCompiler {
    pub fn compile(json: serde_json::Value) -> Result<ExecutionPlan> {
        // 1. Parse Raw JSON into Structs
        // (You might want a dedicated DTO for ReactFlow JSON deserialization)
        let nodes_val = json
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or(crate::error::Error::Validation("Missing nodes".into()))?;
        let edges_val = json
            .get("edges")
            .and_then(|v| v.as_array())
            .ok_or(crate::error::Error::Validation("Missing edges".into()))?;

        let mut nodes = Vec::new();
        for n in nodes_val {
            nodes.push(GraphNode {
                id: n["id"].as_str().unwrap().to_string(),
                type_: n["type"].as_str().unwrap_or("default").to_string(),
            });
        }

        let mut edges = Vec::new();
        for e in edges_val {
            edges.push(GraphEdge {
                source: e["source"].as_str().unwrap().to_string(),
                target: e["target"].as_str().unwrap().to_string(),
                source_handle: e["sourceHandle"].as_str().map(|s| s.to_string()),
            });
        }

        // 2. Validate
        GraphValidator::validate(&nodes, &edges)?;

        // 3. Flatten into Execution Plan
        let mut execution_nodes = HashMap::new();
        let mut start_node_id = String::new();

        // Helper to find connections
        // Map<SourceID, Vec<(Handle, TargetID)>>
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

        // Identify Start Node logic again (simplified for compiler)
        let targets: std::collections::HashSet<String> =
            edges.iter().map(|e| e.target.clone()).collect();

        for node in nodes {
            // Determine Step Type
            let step_type = match node.type_.as_str() {
                "terminal" => StepType::Terminal,
                "authenticator" => StepType::Authenticator,
                _ => StepType::Logic, // Default to logic
            };

            // Capture start node
            if !targets.contains(&node.id) {
                start_node_id = node.id.clone();
            }

            // Build Next Map
            let next_map = adjacency.remove(&node.id).unwrap_or_default();

            execution_nodes.insert(
                node.id.clone(),
                ExecutionNode {
                    id: node.id,
                    step_type,
                    next: next_map,
                    config: serde_json::json!({}), // TODO: Extract from node.data.config
                },
            );
        }

        Ok(ExecutionPlan {
            start_node_id,
            nodes: execution_nodes,
        })
    }
}
