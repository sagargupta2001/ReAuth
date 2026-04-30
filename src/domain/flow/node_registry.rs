use crate::domain::execution::StepType;

/// Definition Metadata for the Compiler.
#[derive(Clone)]
pub struct NodeDefinition {
    pub step_type: StepType,
}

pub trait NodeRegistry {
    fn get_definition(&self, key: &str) -> Option<NodeDefinition>;
}
