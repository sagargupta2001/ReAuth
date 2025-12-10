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
            // --- 1. START NODE (Crucial for Validator) ---
            NodeMetadata {
                id: "core.start".to_string(),
                category: "Start".to_string(),
                display_name: "Start Flow".to_string(),
                description: "The entry point of the authentication flow.".to_string(),
                icon: "Play".to_string(), // Ensure 'Play' icon maps in frontend
                inputs: vec![],           // Validator checks: No inputs = Start
                outputs: vec!["next".to_string()],
                config_schema: json!({}),
            },
            // --- 2. AUTHENTICATORS ---
            NodeMetadata {
                id: "core.auth.password".to_string(),
                category: "Authenticator".to_string(),
                display_name: "Username & Password".to_string(),
                description: "Standard login form challenge.".to_string(),
                icon: "Lock".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec![
                    "success".to_string(),
                    "failure".to_string(),
                    "forgot_password".to_string(),
                ], // Added 3rd output
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "max_attempts": { "type": "integer", "default": 3, "minimum": 1 },
                        "lockout_duration_minutes": { "type": "integer", "default": 15 }
                    }
                }),
            },
            NodeMetadata {
                id: "core.auth.otp".to_string(),
                category: "Authenticator".to_string(),
                display_name: "One-Time Password".to_string(),
                description: "Email or SMS verification code.".to_string(),
                icon: "Smartphone".to_string(), // Ensure icon exists
                inputs: vec!["in".to_string()],
                outputs: vec![
                    "success".to_string(),
                    "failure".to_string(),
                    "resend".to_string(),
                ],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "length": { "type": "integer", "default": 6 },
                        "ttl_seconds": { "type": "integer", "default": 300 },
                        "channel": { "type": "string", "enum": ["email", "sms"], "default": "email" }
                    }
                }),
            },
            // --- 3. LOGIC & BRANCHING ---
            NodeMetadata {
                id: "core.logic.condition".to_string(),
                category: "Logic".to_string(),
                display_name: "Condition Check".to_string(),
                description: "Branch flow based on user or session data.".to_string(),
                icon: "Split".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec!["true".to_string(), "false".to_string()],
                config_schema: json!({
                    "type": "object",
                    "required": ["variable", "operator", "value"],
                    "properties": {
                        "variable": {
                            "type": "string",
                            "description": "e.g. user.email_verified, context.ip_address"
                        },
                        "operator": {
                            "type": "string",
                            "enum": ["equals", "not_equals", "contains", "starts_with"],
                            "default": "equals"
                        },
                        "value": { "type": "string" }
                    }
                }),
            },
            NodeMetadata {
                id: "core.logic.script".to_string(),
                category: "Logic".to_string(),
                display_name: "Execution Script".to_string(),
                description: "Run custom internal logic (e.g. check fraud score).".to_string(),
                icon: "Code".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec!["next".to_string(), "error".to_string()],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "script_name": { "type": "string" }
                    }
                }),
            },
            // --- 4. TERMINALS (Endpoints) ---
            NodeMetadata {
                id: "core.terminal.allow".to_string(),
                category: "Terminal".to_string(),
                display_name: "Allow Access".to_string(),
                description: "Successfully authenticate and issue tokens.".to_string(),
                icon: "CheckCircle".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec![], // No outputs = Terminal
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "issue_refresh_token": { "type": "boolean", "default": true }
                    }
                }),
            },
            NodeMetadata {
                id: "core.terminal.deny".to_string(),
                category: "Terminal".to_string(),
                display_name: "Deny Access".to_string(),
                description: "Reject the authentication attempt.".to_string(),
                icon: "XCircle".to_string(),
                inputs: vec!["in".to_string()],
                outputs: vec![],
                config_schema: json!({
                    "type": "object",
                    "properties": {
                        "error_message": { "type": "string", "default": "Access Denied" },
                        "error_code": { "type": "string", "default": "access_denied" }
                    }
                }),
            },
        ]
    }
}
