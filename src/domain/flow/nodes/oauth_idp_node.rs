use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct OAuthIdpNodeProvider;

impl NodeProvider for OAuthIdpNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.oauth_idp"
    }

    fn display_name(&self) -> &'static str {
        "OAuth Identity Provider"
    }

    fn description(&self) -> &'static str {
        "Redirect the user to a configured external OAuth or OIDC identity provider."
    }

    fn icon(&self) -> &'static str {
        "GlobeLock"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["logged_in", "jit_provisioned", "failed"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "provider_alias": {
                    "type": "string",
                    "title": "Provider Alias",
                    "description": "Optional static provider alias. Leave empty to let the user choose at runtime."
                },
                "template_key": {
                    "type": "string",
                    "title": "Template Key",
                    "default": "oauth_redirecting"
                }
            },
            "additionalProperties": true
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("oauth_redirecting")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
