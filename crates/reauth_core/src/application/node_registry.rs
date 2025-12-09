use crate::domain::flow::NodeMetadata;
use serde_json::json;

pub struct NodeRegistryService {
    // In the future, this will be dynamic (loaded from plugins).
    // For now, we return static core nodes.
}

impl NodeRegistryService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_available_nodes(&self) -> Vec<NodeMetadata> {
        vec![
            NodeMetadata {
                id: "core.auth.password".to_string(),
                category: "Authenticator".to_string(),
                display_name: "Password Form".to_string(),
                description: "Standard Username & Password Challenge".to_string(),
                icon: "Lock".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec!["success".to_string(), "failure".to_string()],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "maxAttempts": { "type": "integer", "default": 3 }
                    }
                }),
            },
            NodeMetadata {
                id: "core.logic.condition".to_string(),
                category: "Logic".to_string(),
                display_name: "Condition".to_string(),
                description: "Branch flow based on context variable".to_string(),
                icon: "Split".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec!["true".to_string(), "false".to_string()],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "variable": { "type": "string" },
                        "operator": { "type": "string", "enum": ["eq", "neq", "contains"] },
                        "value": { "type": "string" }
                    }
                }),
            },
            NodeMetadata {
                id: "core.terminal.success".to_string(),
                category: "Terminal".to_string(),
                display_name: "Success".to_string(),
                description: "Successfully authenticate the user".to_string(),
                icon: "CheckCircle".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec![],
                config_schema: json!({}),
            },
            NodeMetadata {
                id: "core.terminal.failure".to_string(),
                category: "Terminal".to_string(),
                display_name: "Failure".to_string(),
                description: "Fail the authentication attempt".to_string(),
                icon: "XCircle".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec![],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    }
                }),
            },
        ]
    }
}
