use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct PasskeyEnrollNodeProvider;

impl NodeProvider for PasskeyEnrollNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.passkey_enroll"
    }

    fn display_name(&self) -> &'static str {
        "Passkey Enroll"
    }

    fn description(&self) -> &'static str {
        "Enroll a new WebAuthn passkey for the authenticated user."
    }

    fn icon(&self) -> &'static str {
        "KeyRound"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "skip", "failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "allow_skip": {
                    "type": "boolean",
                    "title": "Allow Skip",
                    "default": true
                }
            },
            "additionalProperties": true
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("passkey_enroll")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
