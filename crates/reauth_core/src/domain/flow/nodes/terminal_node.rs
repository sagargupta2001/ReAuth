use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

// --- ALLOW NODE ---
pub struct AllowNode;

impl NodeProvider for AllowNode {
    fn id(&self) -> &'static str {
        "core.terminal.allow"
    }
    fn display_name(&self) -> &'static str {
        "Allow Access"
    }
    fn description(&self) -> &'static str {
        "Successfully authenticate and issue tokens."
    }
    fn icon(&self) -> &'static str {
        "CheckCircle"
    }
    fn category(&self) -> &'static str {
        "Terminal"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec![] // No outputs
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "issue_refresh_token": {
                    "type": "boolean",
                    "default": true,
                    "title": "Issue Refresh Token"
                }
            }
        })
    }
}

// --- DENY NODE ---
pub struct DenyNode;

impl NodeProvider for DenyNode {
    fn id(&self) -> &'static str {
        "core.terminal.deny"
    }
    fn display_name(&self) -> &'static str {
        "Deny Access"
    }
    fn description(&self) -> &'static str {
        "Reject the authentication attempt."
    }
    fn icon(&self) -> &'static str {
        "XCircle"
    }
    fn category(&self) -> &'static str {
        "Terminal"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec![]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "error_message": {
                    "type": "string",
                    "default": "Access Denied",
                    "title": "Error Message"
                },
                "error_code": {
                    "type": "string",
                    "default": "access_denied",
                    "title": "Error Code"
                }
            }
        })
    }
}
