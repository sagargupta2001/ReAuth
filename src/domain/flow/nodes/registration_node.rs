use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct RegistrationNodeProvider;

impl NodeProvider for RegistrationNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.register"
    }

    fn display_name(&self) -> &'static str {
        "Register Account"
    }

    fn description(&self) -> &'static str {
        "Create a new user account in the current realm."
    }

    fn icon(&self) -> &'static str {
        "UserPlus"
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
                },
                "allow_when_invited": {
                    "type": "boolean",
                    "title": "Allow When Invited",
                    "default": false,
                    "description": "Allow registration even when realm self-registration is disabled, but only when invitation context is present."
                }
            }
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("register")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
