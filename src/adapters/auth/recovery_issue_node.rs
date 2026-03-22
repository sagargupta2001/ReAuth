use crate::application::user_service::UserService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::Result;
use crate::ports::realm_recovery_settings_repository::RealmRecoverySettingsRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::distr::{Alphanumeric, SampleString};
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

const RECOVERY_TOKEN_LENGTH: usize = 48;

pub struct RecoveryIssueNode {
    user_service: Arc<UserService>,
    recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
}

impl RecoveryIssueNode {
    pub fn new(
        user_service: Arc<UserService>,
        recovery_settings_repo: Arc<dyn RealmRecoverySettingsRepository>,
    ) -> Self {
        Self {
            user_service,
            recovery_settings_repo,
        }
    }

    fn extract_identifier(session: &AuthenticationSession) -> Option<String> {
        session
            .context
            .get("recovery_identifier")
            .or_else(|| session.context.get("email"))
            .or_else(|| session.context.get("username"))
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

#[async_trait]
impl LifecycleNode for RecoveryIssueNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "recovery_issue", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let Some(identifier) = Self::extract_identifier(session) else {
            return Ok(NodeOutcome::FlowFailure {
                reason: "Recovery identifier missing".to_string(),
            });
        };

        let recovery_settings = self
            .recovery_settings_repo
            .find_by_realm_id(&session.realm_id)
            .await?
            .unwrap_or_else(|| RealmRecoverySettings::defaults(session.realm_id));

        let user_id = self
            .user_service
            .find_by_username(&session.realm_id, &identifier)
            .await?
            .map(|user| user.id.to_string());

        let token = Alphanumeric.sample_string(&mut rand::rng(), RECOVERY_TOKEN_LENGTH);
        let expires_at = Utc::now() + Duration::minutes(recovery_settings.token_ttl_minutes.max(1));

        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("error");
            ctx.insert("email".to_string(), json!(identifier));
            ctx.insert("username".to_string(), json!(identifier));
        } else {
            session.context = json!({ "email": identifier, "username": identifier });
        }

        Ok(NodeOutcome::SuspendForAsync {
            action_type: "reset_credentials".to_string(),
            token: token.clone(),
            expires_at,
            resume_node_id: Some("reset-password".to_string()),
            payload: json!({
                "user_id": user_id,
                "identifier": identifier,
                "resume_path": "/forgot-password",
                "resend_path": "/forgot-password",
            }),
            screen: "core.awaiting-action".to_string(),
            context: json!({
                "message": "If an account exists, recovery instructions will be sent. You can also use the token to continue.",
                "resume_token": token,
                "expires_at": expires_at,
                "action_type": "reset_credentials",
                "resume_path": "/forgot-password",
                "email": identifier,
            }),
        })
    }
}
