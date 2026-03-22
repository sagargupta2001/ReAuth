use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct ResetPasswordNodeProvider;

impl NodeProvider for ResetPasswordNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.reset_password"
    }

    fn display_name(&self) -> &'static str {
        "Reset Password"
    }

    fn description(&self) -> &'static str {
        "Set a new password after verifying a recovery token."
    }

    fn icon(&self) -> &'static str {
        "Key"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "min_password_length": {
                    "type": "integer",
                    "title": "Min Password Length",
                    "default": 8,
                    "minimum": 8
                }
            }
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("reset_password")
    }
}
