use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct InvitationUnavailableNodeProvider;

impl NodeProvider for InvitationUnavailableNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.invitation_unavailable"
    }

    fn display_name(&self) -> &'static str {
        "Invitation Unavailable"
    }

    fn description(&self) -> &'static str {
        "Show an invitation-unavailable page for expired, consumed, or invalid invite links."
    }

    fn icon(&self) -> &'static str {
        "AlertTriangle"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "template_key": {
                    "type": "string",
                    "title": "Template Key",
                    "default": "invitation_unavailable"
                },
                "title": {
                    "type": "string",
                    "title": "Title",
                    "default": "Invitation Link Unavailable"
                },
                "expired_message": {
                    "type": "string",
                    "title": "Expired Message",
                    "default": "This invitation link has expired. Ask your administrator to send a new invitation."
                },
                "consumed_message": {
                    "type": "string",
                    "title": "Consumed Message",
                    "default": "This invitation link has already been used."
                },
                "invalid_message": {
                    "type": "string",
                    "title": "Invalid Message",
                    "default": "This invitation link is invalid."
                }
            },
            "additionalProperties": false
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("invitation_unavailable")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![
            PageCategory::Error,
            PageCategory::Notification,
            PageCategory::Custom,
        ]
    }
}
