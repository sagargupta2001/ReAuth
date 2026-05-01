use crate::domain::execution::StepType;
use crate::domain::flow::node_registry::NodeRegistry;
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
        registry: &dyn NodeRegistry,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::execution::StepType;
    use crate::domain::flow::node_registry::{NodeDefinition, NodeRegistry};
    use crate::error::Error;
    use std::collections::HashMap;

    struct MockRegistry {
        defs: HashMap<String, NodeDefinition>,
    }

    impl NodeRegistry for MockRegistry {
        fn get_definition(&self, key: &str) -> Option<NodeDefinition> {
            self.defs.get(key).cloned()
        }
    }

    #[test]
    fn valid_graph_passes_validation() {
        let nodes = vec![
            GraphNode {
                id: "start".to_string(),
                type_: "core.start".to_string(),
            },
            GraphNode {
                id: "end".to_string(),
                type_: "core.end".to_string(),
            },
        ];
        let edges = vec![GraphEdge {
            source: "start".to_string(),
            target: "end".to_string(),
            source_handle: None,
        }];

        let mut registry = MockRegistry {
            defs: HashMap::new(),
        };
        registry.defs.insert(
            "core.start".to_string(),
            NodeDefinition {
                step_type: StepType::Logic,
            },
        );
        registry.defs.insert(
            "core.end".to_string(),
            NodeDefinition {
                step_type: StepType::Terminal,
            },
        );

        GraphValidator::validate(&nodes, &edges, &registry).unwrap();
    }

    #[test]
    fn dead_end_nodes_are_rejected() {
        let nodes = vec![GraphNode {
            id: "orphan".to_string(),
            type_: "core.logic".to_string(),
        }];
        let edges = Vec::new();

        let mut registry = MockRegistry {
            defs: HashMap::new(),
        };
        registry.defs.insert(
            "core.logic".to_string(),
            NodeDefinition {
                step_type: StepType::Logic,
            },
        );

        let err = GraphValidator::validate(&nodes, &edges, &registry).unwrap_err();
        match err {
            Error::Validation(message) => {
                assert!(message.contains("Dead end"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
