use crate::domain::execution::StepType;
use crate::ports::authenticator::Authenticator;
use std::collections::HashMap;
use std::sync::Arc;

/// Metadata about a node type (e.g. "Is this a Terminal node?")
#[derive(Clone)]
pub struct NodeDefinition {
    pub step_type: StepType,
}

pub struct RuntimeRegistry {
    // Maps "core.auth.password" -> The Rust implementation
    authenticators: HashMap<String, Arc<dyn Authenticator>>,

    // Maps "core.terminal.allow" -> Metadata (StepType::Terminal)
    node_definitions: HashMap<String, NodeDefinition>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            authenticators: HashMap::new(),
            node_definitions: HashMap::new(),
        };

        // --- REGISTER BUILT-IN NODE TYPES ---
        // This is what fixes the "Dead end detected" error.

        // Terminal Nodes (End of flow, no edges required)
        registry.register_node_type("core.terminal.allow", StepType::Terminal);
        registry.register_node_type("core.terminal.deny", StepType::Terminal);

        // Start Node (Logic type)
        registry.register_node_type("core.start", StepType::Logic);

        registry
    }

    /// Register the implementation (The Worker)
    /// This also automatically registers it as an Authenticator type.
    pub fn register_authenticator(&mut self, key: &str, implementation: Arc<dyn Authenticator>) {
        self.authenticators.insert(key.to_string(), implementation);
        self.register_node_type(key, StepType::Authenticator);
    }

    /// Register metadata (The Definition)
    pub fn register_node_type(&mut self, key: &str, step_type: StepType) {
        self.node_definitions
            .insert(key.to_string(), NodeDefinition { step_type });
    }

    pub fn get_authenticator(&self, key: &str) -> Option<Arc<dyn Authenticator>> {
        self.authenticators.get(key).cloned()
    }

    /// Used by FlowValidator to check if a node is Terminal/Logic/Auth
    pub fn get_node_type(&self, key: &str) -> Option<StepType> {
        self.node_definitions.get(key).map(|d| d.step_type.clone())
    }
}
