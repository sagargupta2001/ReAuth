use crate::error::{Error, Result};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub type_: String,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
}

pub struct GraphValidator;

impl GraphValidator {
    pub fn validate(nodes: &[GraphNode], edges: &[GraphEdge]) -> Result<()> {
        if nodes.is_empty() {
            return Err(Error::Validation("Flow cannot be empty".into()));
        }

        // 1. Check for Start Node
        // (Assuming you have a specific type 'start' or we infer it by having 0 incoming edges)
        let targets: HashSet<String> = edges.iter().map(|e| e.target.clone()).collect();
        let starts: Vec<&GraphNode> = nodes.iter().filter(|n| !targets.contains(&n.id)).collect();

        if starts.is_empty() {
            return Err(Error::Validation(
                "Cycle detected: No start node found (all nodes have inputs)".into(),
            ));
        }
        if starts.len() > 1 {
            return Err(Error::Validation(
                "Ambiguous flow: Multiple start nodes detected".into(),
            ));
        }

        // 2. Check for Dead Ends (Nodes that are not Terminal but have no outputs)
        let sources: HashSet<String> = edges.iter().map(|e| e.source.clone()).collect();
        for node in nodes {
            if node.type_ != "terminal" && !sources.contains(&node.id) {
                return Err(Error::Validation(format!(
                    "Dead end detected at node '{}'. All paths must end at a Terminal node.",
                    node.id
                )));
            }
        }

        // 3. (Advanced) Cycle Detection could go here (DFS traversal)

        Ok(())
    }
}
