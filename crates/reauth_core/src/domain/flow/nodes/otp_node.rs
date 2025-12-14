use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct OtpNode;

impl NodeProvider for OtpNode {
    fn id(&self) -> &'static str {
        "core.auth.otp"
    }
    fn display_name(&self) -> &'static str {
        "One-Time Password"
    }
    fn description(&self) -> &'static str {
        "Email or SMS verification code."
    }
    fn icon(&self) -> &'static str {
        "Smartphone"
    }
    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "failure", "resend"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "length": { "type": "integer", "default": 6, "title": "Code Length" },
                "ttl_seconds": { "type": "integer", "default": 300, "title": "TTL (Seconds)" },
                "channel": {
                    "type": "string",
                    "enum": ["email", "sms"],
                    "default": "email",
                    "title": "Delivery Channel"
                }
            }
        })
    }
}
