use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct ForgotCredentialsNodeProvider;

impl NodeProvider for ForgotCredentialsNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.forgot_credentials"
    }

    fn display_name(&self) -> &'static str {
        "Forgot Credentials"
    }

    fn description(&self) -> &'static str {
        "Collect an account identifier to start credential recovery."
    }

    fn icon(&self) -> &'static str {
        "Mail"
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
            "properties": {}
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("forgot_credentials")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
