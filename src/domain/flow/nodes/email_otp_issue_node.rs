use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct EmailOtpIssueNodeProvider;

impl NodeProvider for EmailOtpIssueNodeProvider {
    fn id(&self) -> &'static str {
        "core.logic.issue_email_otp"
    }

    fn display_name(&self) -> &'static str {
        "Issue Email OTP"
    }

    fn description(&self) -> &'static str {
        "Generate an email verification OTP and suspend the flow for async resume."
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
                "identifier_key": {
                    "type": "string",
                    "title": "Identifier Context Key",
                    "default": "email"
                },
                "token_ttl_minutes": {
                    "type": "integer",
                    "title": "Token TTL (minutes)",
                    "minimum": 1,
                    "default": 10
                },
                "resume_path": {
                    "type": "string",
                    "title": "Resume Path",
                    "default": "/register"
                },
                "resend_path": {
                    "type": "string",
                    "title": "Resend Path",
                    "default": "/register"
                },
                "resume_node_id": {
                    "type": "string",
                    "title": "Resume Node ID",
                    "default": "verify-email-otp"
                },
                "email_subject": {
                    "type": "string",
                    "title": "Email Subject"
                },
                "email_body": {
                    "type": "string",
                    "title": "Email Body"
                }
            },
            "additionalProperties": false
        })
    }
}
