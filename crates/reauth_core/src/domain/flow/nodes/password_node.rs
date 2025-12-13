use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct PasswordNode;

impl NodeProvider for PasswordNode {
    fn id(&self) -> &'static str {
        "core.auth.password"
    }
    fn display_name(&self) -> &'static str {
        "Username & Password"
    }
    fn description(&self) -> &'static str {
        "Standard login form challenge."
    }
    fn icon(&self) -> &'static str {
        "Lock"
    }
    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "failure", "forgot_password"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "max_attempts": {
                    "type": "integer",
                    "title": "Max Attempts",
                    "default": 3,
                    "minimum": 1
                },
                "lockout_minutes": {
                    "type": "integer",
                    "title": "Lockout Duration (min)",
                    "default": 15
                },
                "forgot_password_enabled": {
                    "type": "boolean",
                    "title": "Enable Forgot Password",
                    "default": true
                }
            },
            "required": ["max_attempts"]
        })
    }
}
