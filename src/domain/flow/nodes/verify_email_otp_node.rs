use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct VerifyEmailOtpNodeProvider;

impl NodeProvider for VerifyEmailOtpNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.verify_email_otp"
    }

    fn display_name(&self) -> &'static str {
        "Verify Email OTP"
    }

    fn description(&self) -> &'static str {
        "Confirm an email verification token and continue the flow."
    }

    fn icon(&self) -> &'static str {
        "CheckCircle"
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
            "properties": {
                "auto_continue": {
                    "type": "boolean",
                    "title": "Auto-continue after verification",
                    "default": true
                }
            },
            "additionalProperties": false
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn default_template_key(&self) -> Option<&'static str> {
        Some("verify_email")
    }
}
