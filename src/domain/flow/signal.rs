use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ALLOWED_SIGNAL_TYPES: &[&str] = &[
    "submit_node",
    "validate_node",
    "call_subflow",
    "execute_script",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlowSignalEnvelope {
    pub signal: FlowSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlowSignal {
    #[serde(rename = "type")]
    pub signal_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

impl FlowSignal {
    pub fn is_allowed_type(&self) -> bool {
        is_allowed_signal_type(&self.signal_type)
    }

    pub fn normalized_node_id(&self) -> Option<&str> {
        self.node_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }
}

pub fn is_allowed_signal_type(value: &str) -> bool {
    ALLOWED_SIGNAL_TYPES.iter().any(|allowed| allowed == &value)
}
