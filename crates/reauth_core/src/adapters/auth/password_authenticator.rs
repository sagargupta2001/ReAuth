use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{instrument, warn};

use crate::domain::auth_session::AuthenticationSession;
use crate::domain::{
    crypto::HashedPassword,
    execution::lifecycle::{LifecycleNode, NodeOutcome},
};
use crate::error::{Error, Result};
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::user_repository::UserRepository;
const LOCKOUT_MESSAGE: &str = "Account temporarily locked. Try again later.";

/// The Runtime Worker.
/// It implements the LifecycleNode trait to handle the state machine logic.
pub struct PasswordAuthenticator {
    user_repo: Arc<dyn UserRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    lockout_threshold: i64,
    lockout_duration_secs: i64,
}

impl PasswordAuthenticator {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        login_attempt_repo: Arc<dyn LoginAttemptRepository>,
        lockout_threshold: i64,
        lockout_duration_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            realm_repo,
            login_attempt_repo,
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

        // B. Clean Context (Remove errors, keep username, ensure no password leaks)
        if let Some(ctx) = _session.context.as_object_mut() {
            ctx.remove("error");
            ctx.remove("password");
            ctx.insert("username".to_string(), json!(username));
        } else {
            _session.context = json!({ "username": username });
        }

        // C. Move to the "success" edge
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
