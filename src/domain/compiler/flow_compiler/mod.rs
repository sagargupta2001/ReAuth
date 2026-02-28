use super::validator::{GraphEdge, GraphNode, GraphValidator};
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::{ExecutionNode, ExecutionPlan};
use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};

pub struct FlowCompiler;

impl FlowCompiler {
    pub fn compile(json: serde_json::Value, registry: &RuntimeRegistry) -> Result<ExecutionPlan> {
        // 1. Parse Raw JSON
        let nodes_val = json
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or(Error::Validation("Missing nodes".into()))?;
        let edges_val = json
            .get("edges")
            .and_then(|v| v.as_array())
            .ok_or(Error::Validation("Missing edges".into()))?;

        let mut nodes = Vec::new();
        let mut node_configs = HashMap::new();

        for n in nodes_val {
            let id = n["id"].as_str().unwrap().to_string();
            let type_ = n["type"].as_str().unwrap_or("default").to_string();

            // Extract UI Config (node.data.config)
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

        // 2. Validate
        GraphValidator::validate(&nodes, &edges, registry)?;

        // 3. Compile Execution Plan
        let mut execution_nodes = HashMap::new();
        let mut start_node_id = String::new();
        let mut adjacency: HashMap<String, HashMap<String, String>> = HashMap::new();

        // Build Adjacency List
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

        // Identify Start Node
        let targets: HashSet<String> = edges.iter().map(|e| e.target.clone()).collect();

        for node in nodes {
            // Get Definition for StepType
            let def = registry
                .get_definition(&node.type_)
                .ok_or_else(|| Error::Validation(format!("Unknown node type: {}", node.type_)))?;

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
                    step_type: def.step_type,
                    next: next_map,
                    config,
                },
            );
        }

        Ok(ExecutionPlan {
            start_node_id,
            nodes: execution_nodes,
        })
    }
}

#[cfg(test)]
mod tests;
