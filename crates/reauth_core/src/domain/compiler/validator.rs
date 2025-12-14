use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::execution::StepType;
use crate::error::{Error, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
}

pub struct GraphValidator;

impl GraphValidator {
    pub fn validate(
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        registry: &RuntimeRegistry,
    ) -> Result<()> {
        if nodes.is_empty() {
            return Err(Error::Validation("Flow cannot be empty".into()));
        }

        // 1. Check for Start Node (Nodes with no incoming edges)
        let targets: HashSet<String> = edges.iter().map(|e| e.target.clone()).collect();
        let starts: Vec<&GraphNode> = nodes.iter().filter(|n| !targets.contains(&n.id)).collect();

        if starts.is_empty() {
            return Err(Error::Validation(
                "Cycle detected: No start node found".into(),
            ));
        }
        if starts.len() > 1 {
            return Err(Error::Validation(
                "Ambiguous flow: Multiple start nodes detected".into(),
            ));
        }

        // 2. Validate Nodes & Dead Ends
        let sources: HashSet<String> = edges.iter().map(|e| e.source.clone()).collect();

        for node in nodes {
            // Lookup StepType in Registry
            let def = registry
                .get_definition(&node.type_)
                .ok_or_else(|| Error::Validation(format!("Unknown node type: '{}'", node.type_)))?;

            // Terminal nodes are allowed to have no outputs.
            // All other nodes MUST have at least one outgoing edge.
            if def.step_type != StepType::Terminal && !sources.contains(&node.id) {
                return Err(Error::Validation(format!(
                    "Dead end detected at node '{}' ({})",
                    node.id, node.type_
                )));
            }
        }

        Ok(())
    }
}
