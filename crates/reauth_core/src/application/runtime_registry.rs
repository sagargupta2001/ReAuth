use crate::domain::execution::{lifecycle::LifecycleNode, StepType};
use std::collections::HashMap;
use std::sync::Arc;

/// Definition Metadata for the Compiler.
#[derive(Clone)]
pub struct NodeDefinition {
    pub step_type: StepType,
}

pub struct RuntimeRegistry {
    /// The Workers (Executables)
    nodes: HashMap<String, Arc<dyn LifecycleNode>>,

    /// The Blueprints (Metadata)
    definitions: HashMap<String, NodeDefinition>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    /// Registers a functional node (Authenticator, Action)
    pub fn register_node(
        &mut self,
        key: &str,
        implementation: Arc<dyn LifecycleNode>,
        step_type: StepType,
    ) {
        self.nodes.insert(key.to_string(), implementation);
        self.definitions
            .insert(key.to_string(), NodeDefinition { step_type });
    }

    /// Registers a system node (Start, Terminal) that has no custom logic
    /// but is needed for graph validation.
    pub fn register_definition(&mut self, key: &str, step_type: StepType) {
        self.definitions
            .insert(key.to_string(), NodeDefinition { step_type });
    }

    pub fn get_node(&self, key: &str) -> Option<Arc<dyn LifecycleNode>> {
        self.nodes.get(key).cloned()
    }

    pub fn get_definition(&self, key: &str) -> Option<NodeDefinition> {
        self.definitions.get(key).cloned()
    }
}
