use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{instrument, warn};

use crate::application::idp_service::{IdentityProviderLoginOption, IdentityProviderService};
use crate::application::oauth_broker_service::OAuthBrokerService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::{
    crypto::HashedPassword,
    execution::lifecycle::{LifecycleNode, NodeOutcome},
    identity_provider::OAuthBrokerResult,
};
use crate::error::{Error, Result};
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::user_repository::UserRepository;
const LOCKOUT_MESSAGE: &str = "Account temporarily locked. Try again later.";
const LINK_ERROR_KEY: &str = "oauth_link_error";
const FAILURE_KEY: &str = "oauth_failure";

/// The Runtime Worker.
/// It implements the LifecycleNode trait to handle the state machine logic.
pub struct PasswordAuthenticator {
    user_repo: Arc<dyn UserRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    identity_provider_service: Arc<IdentityProviderService>,
    oauth_broker_service: Arc<OAuthBrokerService>,
    lockout_threshold: i64,
    lockout_duration_secs: i64,
}

impl PasswordAuthenticator {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        login_attempt_repo: Arc<dyn LoginAttemptRepository>,
        identity_provider_service: Arc<IdentityProviderService>,
        oauth_broker_service: Arc<OAuthBrokerService>,
        lockout_threshold: i64,
        lockout_duration_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            realm_repo,
            login_attempt_repo,
            identity_provider_service,
            oauth_broker_service,
            lockout_threshold,
            lockout_duration_secs,
        }
    }

    async fn lockout_policy(&self, realm_id: &uuid::Uuid) -> Result<(i64, i64)> {
        if let Some(realm) = self.realm_repo.find_by_id(realm_id).await? {
            return Ok((realm.lockout_threshold, realm.lockout_duration_secs));
        }
        Ok((self.lockout_threshold, self.lockout_duration_secs))
    }

    fn lockout_enabled(&self, threshold: i64, duration_secs: i64) -> bool {
        threshold > 0 && duration_secs > 0
    }

    fn read_broker_result(session: &AuthenticationSession) -> Result<Option<OAuthBrokerResult>> {
        let Some(value) = session.context.get("oauth_broker_result").cloned() else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_value(value).map_err(|e| {
            Error::System(format!("Invalid OAuth broker result: {}", e))
        })?))
    }

    fn clear_transient_state(session: &mut AuthenticationSession) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove(LINK_ERROR_KEY);
            map.remove(FAILURE_KEY);
        }
    }

    fn clear_broker_result(session: &mut AuthenticationSession) {
        if let Some(map) = session.context.as_object_mut() {
            map.remove("oauth_broker_result");
        }
    }

    fn build_oauth_suspend_context(
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

    fn to_oauth_user_context(result: &OAuthBrokerResult) -> Result<Value> {
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

#[async_trait]
impl LifecycleNode for PasswordAuthenticator {
    /// Phase 1: Preparation
    /// Runs when the user *arrives* at this node.
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "password_authenticator",
            phase = "on_enter"
        )
    )]
    async fn on_enter(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // Future: Check Rate Limiter here (e.g. IP block).
        Ok(())
    }

    /// Phase 2: Execution (The Decision)
    /// Decides if we pause for UI or proceed.
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "password_authenticator", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let enabled_providers = self
            .identity_provider_service
            .list_enabled_login_options(session.realm_id)
            .await?;

        if let Some(failure) = session.context.get(FAILURE_KEY).cloned() {
            let provider_alias = failure
                .get("provider_alias")
                .and_then(|value| value.as_str());
            let provider_display_name = provider_alias.and_then(|alias| {
                enabled_providers
                    .iter()
                    .find(|provider| provider.alias == alias)
                    .map(|provider| provider.display_name.as_str())
            });
            let message = failure
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("Sign-in with the external provider failed.");
            return Ok(NodeOutcome::SuspendForUI {
                screen: "core.auth.oauth_idp".to_string(),
                context: Self::build_oauth_suspend_context(
                    "oauth_failure",
                    session.id,
                    &enabled_providers,
                    provider_alias,
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
                    context: Self::build_oauth_suspend_context(
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
                    context: Self::build_oauth_suspend_context(
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

        // Retrieve any error from a previous failed attempt (stored in handle_input)
        let previous_error = session.context.get("error").cloned();

        // Retrieve username if we already know it (pre-fill)
        let username_prefill = session.context.get("username").cloned();

        // Tell the Executor to stop and send this JSON to the UI
        Ok(NodeOutcome::SuspendForUI {
            screen: "login-password".to_string(), // Matches your React Route/Component ID
            context: json!({
                "username": username_prefill,
                "error": previous_error,
                // We can pass flags to the UI here
                "forgotPassword": true
            }),
        })
    }

    /// Phase 3: Handling Input
    /// Runs when the user POSTs data to this node.
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "password_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        _session: &mut AuthenticationSession,
        _input: Value,
    ) -> Result<NodeOutcome> {
        if _input
            .get("oauth_callback")
            .and_then(|value| value.as_bool())
            == Some(true)
        {
            let broker_result = Self::read_broker_result(_session)?
                .ok_or_else(|| Error::Validation("OAuth broker result is missing".to_string()))?;

            match broker_result.output.as_str() {
                "logged_in" | "jit_provisioned" => {
                    _session.user_id = broker_result.user_id;
                    Self::clear_broker_result(_session);
                    Self::clear_transient_state(_session);
                    _session.update_context("oauth", Self::to_oauth_user_context(&broker_result)?);
                    return Ok(NodeOutcome::Continue {
                        output: "success".to_string(),
                    });
                }
                "link_required" | "conflict" => {
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

        if let Some(broker_result) = Self::read_broker_result(_session)? {
            let decision = _input
                .get("decision")
                .and_then(|value| value.as_str())
                .unwrap_or_default();

            if broker_result.output == "link_required" {
                if decision == "cancel" {
                    Self::clear_broker_result(_session);
                    Self::clear_transient_state(_session);
                    return Ok(NodeOutcome::Reject {
                        error: "oauth_link_cancelled".to_string(),
                    });
                }

                let username = _input
                    .get("username")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Validation("Username is required".to_string()))?;

                let password = _input
                    .get("password")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Validation("Password is required".to_string()))?;

                match self
                    .oauth_broker_service
                    .complete_manual_link(_session.realm_id, &broker_result, username, password)
                    .await
                {
                    Ok(linked_result) => {
                        _session.user_id = linked_result.user_id;
                        Self::clear_broker_result(_session);
                        Self::clear_transient_state(_session);
                        _session
                            .update_context("oauth", Self::to_oauth_user_context(&linked_result)?);
                        return Ok(NodeOutcome::Continue {
                            output: "success".to_string(),
                        });
                    }
                    Err(err @ Error::InvalidCredentials) | Err(err @ Error::Validation(_)) => {
                        _session.update_context(LINK_ERROR_KEY, json!(err.to_string()));
                        return Ok(NodeOutcome::Reject {
                            error: err.to_string(),
                        });
                    }
                    Err(err) => return Err(err),
                }
            }

            if broker_result.output == "conflict" && (decision == "retry" || decision == "cancel") {
                Self::clear_broker_result(_session);
                Self::clear_transient_state(_session);
                return Ok(NodeOutcome::Reject {
                    error: "oauth_conflict_retry".to_string(),
                });
            }
        }

        // 1. Extract Credentials
        let username = _input
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or(Error::Validation("Username is required".to_string()))?;

        let password = _input
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or(Error::Validation("Password is required".to_string()))?;

        let (lockout_threshold, lockout_duration_secs) =
            self.lockout_policy(&_session.realm_id).await?;
        let lockout_enabled = self.lockout_enabled(lockout_threshold, lockout_duration_secs);
        if lockout_enabled {
            if let Some(attempt) = self
                .login_attempt_repo
                .find(&_session.realm_id, username)
                .await?
            {
                if let Some(locked_until) = attempt.locked_until {
                    if locked_until > Utc::now() {
                        return self.reject_auth(_session, username, LOCKOUT_MESSAGE).await;
                    }
                    self.login_attempt_repo
                        .clear(&_session.realm_id, username)
                        .await?;
                }
            }
        }

        // 2. Lookup User
        let user = match self
            .user_repo
            .find_by_username(&_session.realm_id, username)
            .await?
        {
            Some(u) => u,
            None => {
                // Security: Fake verify to prevent timing attacks (optional)
                warn!("Login failed: User not found '{}'", username);
                if lockout_enabled {
                    let attempt = self
                        .login_attempt_repo
                        .record_failure(
                            &_session.realm_id,
                            username,
                            lockout_threshold,
                            lockout_duration_secs,
                        )
                        .await?;
                    if let Some(locked_until) = attempt.locked_until {
                        if locked_until > Utc::now() {
                            return self.reject_auth(_session, username, LOCKOUT_MESSAGE).await;
                        }
                    }
                }
                return self
                    .reject_auth(_session, username, "Invalid credentials")
                    .await;
            }
        };

        if user.password_login_disabled {
            return self
                .reject_auth(
                    _session,
                    username,
                    "Password login is disabled for this account. Use a passkey.",
                )
                .await;
        }

        // 3. Verify Password Hash
        let hashed = HashedPassword::from_hash(&user.hashed_password)?;
        if !hashed.verify(password)? {
            warn!("Login failed: Invalid password for '{}'", username);
            if lockout_enabled {
                let attempt = self
                    .login_attempt_repo
                    .record_failure(
                        &_session.realm_id,
                        username,
                        lockout_threshold,
                        lockout_duration_secs,
                    )
                    .await?;
                if let Some(locked_until) = attempt.locked_until {
                    if locked_until > Utc::now() {
                        return self.reject_auth(_session, username, LOCKOUT_MESSAGE).await;
                    }
                }
            }
            return self
                .reject_auth(_session, username, "Invalid credentials")
                .await;
        }

        if lockout_enabled {
            self.login_attempt_repo
                .clear(&_session.realm_id, username)
                .await?;
        }

        // 4. Success Logic
        // A. Update Identity in Session
        _session.user_id = Some(user.id);
        _session.update_context("user_id", Value::String(user.id.to_string()));

        // B. Clean Context (Remove errors, keep username, ensure no password leaks)
        if let Some(ctx) = _session.context.as_object_mut() {
            ctx.remove("error");
            ctx.remove("password");
            ctx.insert("username".to_string(), json!(username));
        } else {
            _session.context = json!({ "username": username });
        }

        // C. Move to the next edge
        if user.force_password_reset {
            if let Some(ctx) = _session.context.as_object_mut() {
                ctx.insert("force_password_reset".to_string(), json!(true));
            }
            return Ok(NodeOutcome::Continue {
                output: "force_reset".to_string(),
            });
        }

        Ok(NodeOutcome::Continue {
            output: "success".to_string(),
        })
    }

    /// Phase 4: Exit Cleanup
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "password_authenticator", phase = "on_exit")
    )]
    async fn on_exit(&self, _session: &mut AuthenticationSession) -> Result<()> {
        // Paranoid cleanup: Ensure password never lingers
        if let Some(ctx) = _session.context.as_object_mut() {
            ctx.remove("password");
        }
        Ok(())
    }
}

impl PasswordAuthenticator {
    /// Helper to handle rejection state updates
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "password_authenticator", phase = "reject")
    )]
    async fn reject_auth(
        &self,
        session: &mut AuthenticationSession,
        username: &str,
        reason: &str,
    ) -> Result<NodeOutcome> {
        // Store the error in context so `execute` can display it on re-render
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("error".to_string(), json!(reason));
            ctx.insert("username".to_string(), json!(username));
        } else {
            session.context = json!({
                "error": reason,
                "username": username
            });
        }

        // Return Reject to stay on the same node
        Ok(NodeOutcome::Reject {
            error: reason.to_string(),
        })
    }
}
