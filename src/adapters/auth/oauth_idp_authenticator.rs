use crate::application::idp_service::{IdentityProviderLoginOption, IdentityProviderService};
use crate::application::oauth_broker_service::OAuthBrokerService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::domain::identity_provider::OAuthBrokerResult;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

const SELECTED_PROVIDER_ALIAS_KEY: &str = "oauth_selected_provider_alias";
const LINK_ERROR_KEY: &str = "oauth_link_error";
const FAILURE_KEY: &str = "oauth_failure";

impl OAuthIdpAuthenticator {
    pub fn new(
        identity_provider_service: Arc<IdentityProviderService>,
        oauth_broker_service: Arc<OAuthBrokerService>,
    ) -> Self {
        Self {
            identity_provider_service,
            oauth_broker_service,
        }
    }

    fn node_config(session: &AuthenticationSession) -> Value {
        session
            .context
            .get("node_config")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    fn selected_provider_alias(session: &AuthenticationSession) -> Option<String> {
        session
            .context
            .get(SELECTED_PROVIDER_ALIAS_KEY)
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .or_else(|| {
                Self::node_config(session)
                    .get("provider_alias")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
            })
    }

    fn read_broker_result(session: &AuthenticationSession) -> Result<Option<OAuthBrokerResult>> {
        let Some(value) = session.context.get("oauth_broker_result").cloned() else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_value(value).map_err(|e| {
            Error::System(format!("Invalid OAuth broker result: {}", e))
        })?))
    }

    fn clear_transient_state(session: &mut AuthenticationSession, clear_provider_alias: bool) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove(LINK_ERROR_KEY);
            map.remove(FAILURE_KEY);
            if clear_provider_alias {
                map.remove(SELECTED_PROVIDER_ALIAS_KEY);
            }
        }
    }

    fn clear_broker_result(session: &mut AuthenticationSession) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove("oauth_broker_result");
        }
    }

    fn set_failure_state(
        session: &mut AuthenticationSession,
        message: impl Into<String>,
        provider_alias: Option<&str>,
    ) {
        session.update_context(
            FAILURE_KEY,
            json!({
                "message": message.into(),
                "provider_alias": provider_alias
            }),
        );
    }

    fn build_suspend_context(
        template_key: &str,
        auth_session_id: uuid::Uuid,
        enabled_providers: &[IdentityProviderLoginOption],
        provider_alias: Option<&str>,
        provider_display_name: Option<&str>,
        extra: Value,
    ) -> Value {
        let mut base = json!({
            "template_key": template_key,
            "auth_session_id": auth_session_id,
            "enabled_providers": enabled_providers,
            "enabled_providers_count": enabled_providers.len(),
            "provider_alias": provider_alias,
            "provider_display_name": provider_display_name
        });
        if let (Some(base_map), Some(extra_map)) = (base.as_object_mut(), extra.as_object()) {
            for (key, value) in extra_map {
                base_map.insert(key.clone(), value.clone());
            }
        }
        base
    }

    fn to_user_context(result: &OAuthBrokerResult) -> Result<Value> {
        let user_id = result
            .user_id
            .ok_or_else(|| Error::System("OAuth broker result is missing user_id".to_string()))?;
        Ok(json!({
            "provider_id": result.provider_id,
            "provider_alias": result.provider_alias,
            "provider_display_name": result.provider_display_name,
            "subject": result.subject,
            "external_email": result.external_email,
            "external_username": result.external_username,
            "user_id": user_id
        }))
    }
}

pub struct OAuthIdpAuthenticator {
    identity_provider_service: Arc<IdentityProviderService>,
    oauth_broker_service: Arc<OAuthBrokerService>,
}

#[async_trait]
impl LifecycleNode for OAuthIdpAuthenticator {
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let enabled_providers = self
            .identity_provider_service
            .list_enabled_login_options(session.realm_id)
            .await?;
        let selected_provider_alias = Self::selected_provider_alias(session);
        let provider_display_name = selected_provider_alias.as_deref().and_then(|alias| {
            enabled_providers
                .iter()
                .find(|provider| provider.alias == alias)
                .map(|provider| provider.display_name.as_str())
        });

        if let Some(failure) = session.context.get(FAILURE_KEY).cloned() {
            let message = failure
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("Sign-in with the external provider failed.");
            return Ok(NodeOutcome::SuspendForUI {
                screen: "core.auth.oauth_idp".to_string(),
                context: Self::build_suspend_context(
                    "oauth_failure",
                    session.id,
                    &enabled_providers,
                    selected_provider_alias.as_deref(),
                    provider_display_name,
                    json!({
                        "error": message,
                        "message": message,
                        "can_retry": true
                    }),
                ),
            });
        }

        if let Some(broker_result) = Self::read_broker_result(session)? {
            if broker_result.output == "link_required" {
                let link_error = session
                    .context
                    .get(LINK_ERROR_KEY)
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                return Ok(NodeOutcome::SuspendForUI {
                    screen: "core.auth.oauth_idp".to_string(),
                    context: Self::build_suspend_context(
                        "oauth_link_confirm",
                        session.id,
                        &enabled_providers,
                        Some(&broker_result.provider_alias),
                        Some(&broker_result.provider_display_name),
                        json!({
                            "message": broker_result.message,
                            "error": link_error,
                            "external_email": broker_result.external_email,
                            "external_username": broker_result.external_username,
                            "username": broker_result.external_email
                        }),
                    ),
                });
            }
            if broker_result.output == "conflict" {
                return Ok(NodeOutcome::SuspendForUI {
                    screen: "core.auth.oauth_idp".to_string(),
                    context: Self::build_suspend_context(
                        "oauth_conflict",
                        session.id,
                        &enabled_providers,
                        Some(&broker_result.provider_alias),
                        Some(&broker_result.provider_display_name),
                        json!({
                            "message": broker_result.message,
                            "external_email": broker_result.external_email,
                            "external_username": broker_result.external_username
                        }),
                    ),
                });
            }
        }

        let Some(provider_alias) = selected_provider_alias else {
            return Ok(NodeOutcome::SuspendForUI {
                screen: "core.auth.oauth_idp".to_string(),
                context: Self::build_suspend_context(
                    "oauth_select",
                    session.id,
                    &enabled_providers,
                    None,
                    None,
                    json!({
                        "message": "Choose a sign-in provider to continue."
                    }),
                ),
            });
        };

        if provider_display_name.is_none() {
            Self::set_failure_state(
                session,
                "The selected identity provider is unavailable.",
                Some(&provider_alias),
            );
            return Ok(NodeOutcome::SuspendForUI {
                screen: "core.auth.oauth_idp".to_string(),
                context: Self::build_suspend_context(
                    "oauth_failure",
                    session.id,
                    &enabled_providers,
                    Some(&provider_alias),
                    None,
                    json!({
                        "error": "The selected identity provider is unavailable.",
                        "message": "The selected identity provider is unavailable.",
                        "can_retry": true
                    }),
                ),
            });
        }

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.oauth_idp".to_string(),
            context: Self::build_suspend_context(
                "oauth_redirecting",
                session.id,
                &enabled_providers,
                Some(&provider_alias),
                provider_display_name,
                json!({
                    "auto_start": true
                }),
            ),
        })
    }

    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        if input
            .get("oauth_callback")
            .and_then(|value| value.as_bool())
            == Some(true)
        {
            let broker_result = Self::read_broker_result(session)?
                .ok_or_else(|| Error::Validation("OAuth broker result is missing".to_string()))?;

            match broker_result.output.as_str() {
                "logged_in" | "jit_provisioned" => {
                    session.user_id = broker_result.user_id;
                    Self::clear_broker_result(session);
                    Self::clear_transient_state(session, true);
                    session.update_context("oauth", Self::to_user_context(&broker_result)?);
                    return Ok(NodeOutcome::Continue {
                        output: broker_result.output,
                    });
                }
                "link_required" | "conflict" => {
                    session.update_context(
                        SELECTED_PROVIDER_ALIAS_KEY,
                        json!(broker_result.provider_alias),
                    );
                    return Ok(NodeOutcome::Reject {
                        error: "oauth_broker_follow_up".to_string(),
                    });
                }
                other => {
                    return Err(Error::Validation(format!(
                        "Unexpected OAuth broker output '{}'",
                        other
                    )));
                }
            }
        }

        if let Some(provider_alias) = input.get("provider_alias").and_then(|value| value.as_str()) {
            let enabled = self
                .identity_provider_service
                .list_enabled_login_options(session.realm_id)
                .await?;
            if !enabled
                .iter()
                .any(|provider| provider.alias == provider_alias)
            {
                Self::set_failure_state(
                    session,
                    "The selected identity provider is unavailable.",
                    Some(provider_alias),
                );
            } else {
                Self::clear_broker_result(session);
                Self::clear_transient_state(session, false);
                session.update_context(SELECTED_PROVIDER_ALIAS_KEY, json!(provider_alias));
            }
            return Ok(NodeOutcome::Reject {
                error: "oauth_provider_selected".to_string(),
            });
        }

        let decision = input
            .get("decision")
            .and_then(|value| value.as_str())
            .unwrap_or_default();

        if let Some(broker_result) = Self::read_broker_result(session)? {
            if broker_result.output == "link_required" {
                if decision == "cancel" {
                    Self::clear_broker_result(session);
                    Self::clear_transient_state(session, true);
                    return Ok(NodeOutcome::Continue {
                        output: "failed".to_string(),
                    });
                }

                let username = input
                    .get("username")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| Error::Validation("Username is required".to_string()))?;
                let password = input
                    .get("password")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| Error::Validation("Password is required".to_string()))?;

                match self
                    .oauth_broker_service
                    .complete_manual_link(session.realm_id, &broker_result, username, password)
                    .await
                {
                    Ok(linked_result) => {
                        session.user_id = linked_result.user_id;
                        Self::clear_broker_result(session);
                        Self::clear_transient_state(session, true);
                        session.update_context("oauth", Self::to_user_context(&linked_result)?);
                        return Ok(NodeOutcome::Continue {
                            output: linked_result.output,
                        });
                    }
                    Err(err @ Error::InvalidCredentials) => {
                        session.update_context(LINK_ERROR_KEY, json!(err.to_string()));
                        return Ok(NodeOutcome::Reject {
                            error: err.to_string(),
                        });
                    }
                    Err(err @ Error::Validation(_)) => {
                        session.update_context(LINK_ERROR_KEY, json!(err.to_string()));
                        return Ok(NodeOutcome::Reject {
                            error: err.to_string(),
                        });
                    }
                    Err(err) => return Err(err),
                }
            }

            if broker_result.output == "conflict" {
                if decision == "retry" {
                    Self::clear_broker_result(session);
                    Self::clear_transient_state(session, true);
                    return Ok(NodeOutcome::Reject {
                        error: "oauth_retry".to_string(),
                    });
                }
                Self::clear_broker_result(session);
                Self::clear_transient_state(session, true);
                return Ok(NodeOutcome::Continue {
                    output: "failed".to_string(),
                });
            }
        }

        if decision == "retry" {
            Self::clear_broker_result(session);
            Self::clear_transient_state(session, true);
            return Ok(NodeOutcome::Reject {
                error: "oauth_retry".to_string(),
            });
        }
        if decision == "cancel" {
            Self::clear_broker_result(session);
            Self::clear_transient_state(session, true);
            return Ok(NodeOutcome::Continue {
                output: "failed".to_string(),
            });
        }

        Err(Error::Validation(
            "OAuth broker input was not recognized".to_string(),
        ))
    }
}
