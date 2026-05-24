use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct CollectIdpChoiceNodeProvider;

impl NodeProvider for CollectIdpChoiceNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.collect_idp_choice"
    }

    fn display_name(&self) -> &'static str {
        "Choose Identity Provider"
    }

    fn description(&self) -> &'static str {
        "Render the provider picker and store the selected OAuth or OIDC provider for the next node."
    }

    fn icon(&self) -> &'static str {
        "ListChecks"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["selected", "failed"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "template_key": {
                    "type": "string",
                    "title": "Template Key",
                    "default": "oauth_select"
                }
            },
            "additionalProperties": true
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("oauth_select")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::Auth]
    }
}
