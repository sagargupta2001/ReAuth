use proptest::prelude::*;
use reauth_core::application::runtime_registry::RuntimeRegistry;
use reauth_core::domain::compiler::validator::{GraphEdge, GraphNode, GraphValidator};
use reauth_core::domain::execution::StepType;
use reauth_core::error::Error;

proptest! {
    #[test]
    fn disconnected_non_terminal_node_is_dead_end(
        num_disconnected in 1..10usize,
    ) {
        let mut nodes = vec![
            GraphNode {
                id: "start".to_string(),
                type_: "core.start".to_string(),
            },
            GraphNode {
                id: "end".to_string(),
                type_: "core.end".to_string(),
            }
        ];

        let mut edges = vec![
            GraphEdge {
                source: "start".to_string(),
                target: "end".to_string(),
                source_handle: None,
            }
        ];

        for i in 0..num_disconnected {
            let id = format!("orphan_{}", i);
            nodes.push(GraphNode {
                id: id.clone(),
                type_: "core.logic".to_string(),
            });
            edges.push(GraphEdge {
                source: "start".to_string(),
                target: id,
                source_handle: None,
            });
        }

        let mut registry = RuntimeRegistry::new();
        registry.register_definition("core.start", StepType::Logic);
        registry.register_definition("core.end", StepType::Terminal);
        registry.register_definition("core.logic", StepType::Logic);

        let res = GraphValidator::validate(&nodes, &edges, &registry);
        prop_assert!(res.is_err());

        if let Err(Error::Validation(msg)) = res {
            prop_assert!(msg.contains("Dead end"));
        } else {
            prop_assert!(false, "Expected validation error");
        }
    }

    #[test]
    fn multiple_start_nodes_rejected(
        num_extra_starts in 1..5usize,
    ) {
        let mut nodes = vec![
            GraphNode {
                id: "start".to_string(),
                type_: "core.start".to_string(),
            },
            GraphNode {
                id: "end".to_string(),
                type_: "core.end".to_string(),
            }
        ];

        let mut edges = vec![
            GraphEdge {
                source: "start".to_string(),
                target: "end".to_string(),
                source_handle: None,
            }
        ];

        for i in 0..num_extra_starts {
            let id = format!("extra_start_{}", i);
            nodes.push(GraphNode {
                id: id.clone(),
                type_: "core.start".to_string(),
            });
            edges.push(GraphEdge {
                source: id,
                target: "end".to_string(),
                source_handle: None,
            });
        }

        let mut registry = RuntimeRegistry::new();
        registry.register_definition("core.start", StepType::Logic);
        registry.register_definition("core.end", StepType::Terminal);

        let res = GraphValidator::validate(&nodes, &edges, &registry);
        prop_assert!(res.is_err());

        if let Err(Error::Validation(msg)) = res {
            prop_assert!(msg.contains("Multiple start nodes"));
        } else {
            prop_assert!(false, "Expected validation error");
        }
    }
}
