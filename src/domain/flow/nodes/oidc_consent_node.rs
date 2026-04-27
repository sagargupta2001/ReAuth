use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct OidcConsentNodeProvider;

impl NodeProvider for OidcConsentNodeProvider {
    fn id(&self) -> &'static str {
        "core.oidc.consent"
    }

    fn display_name(&self) -> &'static str {
        "OIDC Consent"
    }

    fn description(&self) -> &'static str {
        "Request user consent for OIDC scopes before issuing tokens."
    }

    fn icon(&self) -> &'static str {
        "ShieldAlert"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["allow", "deny"]
    }

    fn config_schema(&self) -> Value {
        json!({})
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("consent")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Consent]
    }
}
