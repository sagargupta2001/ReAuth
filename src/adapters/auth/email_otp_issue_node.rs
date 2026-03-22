use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::RngExt;
use serde_json::{json, Value};
use tracing::instrument;

const OTP_LENGTH: usize = 6;
const DEFAULT_TTL_MINUTES: i64 = 10;

pub struct EmailOtpIssueNode;

impl EmailOtpIssueNode {
    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn extract_identifier(session: &AuthenticationSession, config: &Value) -> Option<String> {
        let identifier_key = config
            .get("identifier_key")
            .and_then(|value| value.as_str())
            .unwrap_or("email");
        session
            .context
            .get(identifier_key)
            .or_else(|| session.context.get("email"))
            .or_else(|| session.context.get("username"))
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn resolve_string(config: &Value, key: &str) -> Option<String> {
        config
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn resolve_i64(config: &Value, key: &str, fallback: i64) -> i64 {
        config
            .get(key)
            .and_then(|value| value.as_i64())
            .unwrap_or(fallback)
            .max(1)
    }

    fn generate_token() -> String {
        let mut rng = rand::rng();
        (0..OTP_LENGTH)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    }

    fn resolve_user_id(session: &AuthenticationSession) -> Option<String> {
        session.user_id.map(|value| value.to_string()).or_else(|| {
            session
                .context
                .get("user_id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
    }
}

#[async_trait]
impl LifecycleNode for EmailOtpIssueNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "email_otp_issue_node", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let config = Self::node_config(session);
        let Some(identifier) = Self::extract_identifier(session, &config) else {
            return Ok(NodeOutcome::FlowFailure {
                reason: "Email identifier missing for verification".to_string(),
            });
        };

        let token = Self::generate_token();
        let ttl = Self::resolve_i64(&config, "token_ttl_minutes", DEFAULT_TTL_MINUTES);
        let expires_at = Utc::now() + Duration::minutes(ttl);
        let resume_path =
            Self::resolve_string(&config, "resume_path").unwrap_or_else(|| "/register".to_string());
        let resend_path =
            Self::resolve_string(&config, "resend_path").unwrap_or_else(|| resume_path.clone());
        let resume_node_id = Self::resolve_string(&config, "resume_node_id")
            .or(Some("verify-email-otp".to_string()));
        let email_subject = Self::resolve_string(&config, "email_subject");
        let email_body = Self::resolve_string(&config, "email_body");
        let user_id = Self::resolve_user_id(session);

        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("email".to_string(), json!(identifier));
        } else {
            session.context = json!({ "email": identifier });
        }

        Ok(NodeOutcome::SuspendForAsync {
            action_type: "email_verify".to_string(),
            token: token.clone(),
            expires_at,
            resume_node_id,
            payload: json!({
                "identifier": identifier,
                "user_id": user_id,
                "resume_path": resume_path,
                "resend_path": resend_path,
                "email_subject": email_subject,
                "email_body": email_body,
            }),
            screen: "core.awaiting-action".to_string(),
            context: json!({
                "message": "If an account exists, a verification email has been sent.",
                "resume_token": token,
                "expires_at": expires_at,
                "action_type": "email_verify",
                "resume_path": resume_path,
                "resend_path": resend_path,
                "email": identifier,
            }),
        })
    }
}
