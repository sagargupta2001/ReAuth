use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct PasskeyAssertNodeProvider;

impl NodeProvider for PasskeyAssertNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.passkey_assert"
    }

    fn display_name(&self) -> &'static str {
        "Passkey Assert"
    }

    fn description(&self) -> &'static str {
        "Authenticate a user using WebAuthn passkey assertion before optional password fallback."
    }

    fn icon(&self) -> &'static str {
        "Fingerprint"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "fallback", "failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": {
                    "type": "string",
                    "title": "Intent",
                    "enum": ["login", "reauth"],
                    "default": "login"
                },
                "fallback_output": {
                    "type": "string",
                    "title": "Fallback Output",
                    "default": "fallback"
                }
            },
            "additionalProperties": true
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("passkey_assert")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
