use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::{json, Value};

pub struct InvitationIssueNodeProvider;

impl NodeProvider for InvitationIssueNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.issue_invitation"
    }

    fn display_name(&self) -> &'static str {
        "Issue Invitation Token"
    }

    fn description(&self) -> &'static str {
        "Generate an invitation token and suspend the flow for async email delivery/resume."
    }

    fn icon(&self) -> &'static str {
        "Mail"
    }

    fn category(&self) -> &'static str {
        "Logic"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["default"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["issued"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "logic_type": {
                    "type": "string",
                    "const": "core.logic.issue_invitation",
                    "default": "core.logic.issue_invitation"
                },
                "resume_path": {
                    "type": "string",
                    "title": "Resume Path",
                    "default": "/invite/accept"
                },
                "resend_path": {
                    "type": "string",
                    "title": "Resend Path",
                    "default": "/invite/accept"
                },
                "resume_node_id": {
                    "type": "string",
                    "title": "Resume Node ID",
                    "default": "auth-register"
                }
            },
            "additionalProperties": false
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("awaiting_action")
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::AwaitingAction)
    }

    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        vec![PageCategory::AwaitingAction]
    }
}
