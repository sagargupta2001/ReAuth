use crate::application::idp_service::{IdentityProviderLoginOption, IdentityProviderService};
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

const SELECTED_PROVIDER_ALIAS_KEY: &str = "oauth_selected_provider_alias";
const CHOICE_ERROR_KEY: &str = "oauth_choice_error";

pub struct CollectIdpChoiceAuthenticator {
    identity_provider_service: Arc<IdentityProviderService>,
}

impl CollectIdpChoiceAuthenticator {
    pub fn new(identity_provider_service: Arc<IdentityProviderService>) -> Self {
        Self {
            identity_provider_service,
        }
    }

    fn selected_provider_alias(session: &AuthenticationSession) -> Option<String> {
        session
            .context
            .get(SELECTED_PROVIDER_ALIAS_KEY)
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
    }

    fn clear_error(session: &mut AuthenticationSession) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove(CHOICE_ERROR_KEY);
        }
    }

    fn set_error(session: &mut AuthenticationSession, message: &str) {
        session.update_context(CHOICE_ERROR_KEY, json!(message));
    }

    fn clear_selected_provider(session: &mut AuthenticationSession) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove(SELECTED_PROVIDER_ALIAS_KEY);
        }
    }

    fn build_suspend_context(
        auth_session_id: uuid::Uuid,
        enabled_providers: &[IdentityProviderLoginOption],
        provider_alias: Option<&str>,
        error: Option<&str>,
    ) -> Value {
        json!({
            "template_key": "oauth_select",
            "auth_session_id": auth_session_id,
            "enabled_providers": enabled_providers,
            "enabled_providers_count": enabled_providers.len(),
            "provider_alias": provider_alias,
            "message": "Choose a sign-in provider to continue.",
            "error": error,
        })
    }
}

#[async_trait]
impl LifecycleNode for CollectIdpChoiceAuthenticator {
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let enabled_providers = self
            .identity_provider_service
            .list_enabled_login_options(session.realm_id)
            .await?;
        let provider_alias = Self::selected_provider_alias(session);
        let error = session
            .context
            .get(CHOICE_ERROR_KEY)
            .and_then(|value| value.as_str());

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.collect_idp_choice".to_string(),
            context: Self::build_suspend_context(
                session.id,
                &enabled_providers,
                provider_alias.as_deref(),
                error,
            ),
        })
    }

    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let decision = input
            .get("decision")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        if decision == "cancel" {
            Self::clear_error(session);
            Self::clear_selected_provider(session);
            return Ok(NodeOutcome::Continue {
                output: "failed".to_string(),
            });
        }

        let Some(provider_alias) = input.get("provider_alias").and_then(|value| value.as_str())
        else {
            Self::set_error(session, "Choose an identity provider to continue.");
            return Ok(NodeOutcome::Reject {
                error: "oauth_provider_required".to_string(),
            });
        };

        let enabled_providers = self
            .identity_provider_service
            .list_enabled_login_options(session.realm_id)
            .await?;
        if !enabled_providers
            .iter()
            .any(|provider| provider.alias == provider_alias)
        {
            Self::set_error(session, "The selected identity provider is unavailable.");
            return Ok(NodeOutcome::Reject {
                error: "oauth_provider_unavailable".to_string(),
            });
        }

        Self::clear_error(session);
        session.update_context(SELECTED_PROVIDER_ALIAS_KEY, json!(provider_alias));
        Ok(NodeOutcome::Continue {
            output: "selected".to_string(),
        })
    }
}
