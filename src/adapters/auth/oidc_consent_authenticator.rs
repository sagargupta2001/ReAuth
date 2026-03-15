use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::instrument;

use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::{Error, Result};

pub struct OidcConsentAuthenticator;

impl OidcConsentAuthenticator {
    pub fn new() -> Self {
        Self
    }

    fn build_context(session: &AuthenticationSession) -> Value {
        let oidc = session
            .context
            .get("oidc")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let client_id = oidc
            .get("client_id")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let scopes = oidc
            .get("scope")
            .and_then(|value| value.as_str())
            .map(parse_scopes)
            .unwrap_or_default();
        let error = session.context.get("error").cloned();

        json!({
            "oidc": oidc,
            "client_id": client_id,
            "scopes": scopes,
            "error": error
        })
    }

    fn resolve_decision(input: &Value) -> Option<String> {
        let raw = input
            .get("decision")
            .or_else(|| input.get("action"))
            .or_else(|| input.get("consent"))
            .and_then(|value| value.as_str())?;
        Some(raw.trim().to_lowercase())
    }
}

#[async_trait]
impl LifecycleNode for OidcConsentAuthenticator {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "oidc_consent", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        Ok(NodeOutcome::SuspendForUI {
            screen: "core.oidc.consent".to_string(),
            context: Self::build_context(session),
        })
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "oidc_consent", phase = "handle_input")
    )]
    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let decision = Self::resolve_decision(&input).unwrap_or_else(|| "allow".to_string());
        let output = match decision.as_str() {
            "allow" | "approve" | "yes" | "true" => "allow",
            "deny" | "reject" | "no" | "false" => "deny",
            _ => {
                return Err(Error::Validation(
                    "Consent decision must be allow or deny".to_string(),
                ))
            }
        };

        Ok(NodeOutcome::Continue {
            output: output.to_string(),
        })
    }
}

fn parse_scopes(scope: &str) -> Vec<String> {
    scope
        .split_whitespace()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect()
}
