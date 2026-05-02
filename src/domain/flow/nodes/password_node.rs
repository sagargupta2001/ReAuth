use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

/// The Definition/Metadata for the Password Node.
/// Used by the UI Builder to render the node and config form.
pub struct PasswordNodeProvider;

impl NodeProvider for PasswordNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.password"
    }

    fn display_name(&self) -> &'static str {
        "Username & Password"
    }

    fn description(&self) -> &'static str {
        "Standard login form challenge with password verification."
    }

    fn icon(&self) -> &'static str {
        "Lock"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    // Password authentication can optionally branch into forced reset flows.
    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "force_reset", "failure"]
    }

    // Configuration Schema for the UI Sidebar
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
                "forgot_password_enabled": {
                    "type": "boolean",
                    "title": "Enable Forgot Password",
                    "default": true
                }
            },
            "required": ["max_attempts"]
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("login")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
