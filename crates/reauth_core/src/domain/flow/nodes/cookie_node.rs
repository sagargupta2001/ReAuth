use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct CookieNodeProvider;

impl NodeProvider for CookieNodeProvider {
    fn id(&self) -> &'static str {
        "core.auth.cookie"
    }

    fn display_name(&self) -> &'static str {
        "Cookie / SSO"
    }

    fn description(&self) -> &'static str {
        "Checks for a valid SSO session cookie. If found, logs the user in automatically."
    }

    fn icon(&self) -> &'static str {
        "cookie"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn inputs(&self) -> Vec<&'static str> {
        vec!["default"]
    }

    fn outputs(&self) -> Vec<&'static str> {
        // [FIX] Changed from "default" to "continue" for clarity
        vec!["continue"]
    }
    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }
}
