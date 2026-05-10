use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct InvitationTokenNodeProvider;

impl NodeProvider for InvitationTokenNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.invitation_token"
    }

    fn display_name(&self) -> &'static str {
        "Validate Invitation"
    }

    fn description(&self) -> &'static str {
        "Validates invitation token state and routes by status before account creation."
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
        vec!["valid", "expired", "consumed", "invalid"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "logic_type": {
                    "type": "string",
                    "const": "core.logic.invitation_token",
                    "default": "core.logic.invitation_token"
                }
            },
            "additionalProperties": false
        })
    }
}
