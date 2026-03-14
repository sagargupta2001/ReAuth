use crate::application::user_service::UserService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::instrument;

pub struct RegistrationAuthenticator {
    user_service: Arc<UserService>,
}

impl RegistrationAuthenticator {
    pub fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
}

#[async_trait]
impl LifecycleNode for RegistrationAuthenticator {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "registration_authenticator",
            phase = "execute"
        )
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let previous_error = session.context.get("error").cloned();
        let username_prefill = session.context.get("username").cloned();

        Ok(NodeOutcome::SuspendForUI {
            screen: "core.auth.register".to_string(),
            context: json!({
                "username": username_prefill,
                "error": previous_error,
            }),
        })
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "registration_authenticator",
            phase = "handle_input"
        )
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let username = input
            .get("username")
            .or_else(|| input.get("email"))
            .and_then(|value| value.as_str())
            .ok_or(Error::Validation("Username is required".to_string()))?;

        if username.trim().len() < 3 {
            return self
                .reject_registration(session, username, "Username must be at least 3 characters")
                .await;
        }

        let password = input
            .get("password")
            .and_then(|value| value.as_str())
            .ok_or(Error::Validation("Password is required".to_string()))?;

        if password.len() < 8 {
            return self
                .reject_registration(session, username, "Password must be at least 8 characters")
                .await;
        }

        match self
            .user_service
            .create_user(session.realm_id, username, password)
            .await
        {
            Ok(user) => {
                session.user_id = Some(user.id);
                if let Some(ctx) = session.context.as_object_mut() {
                    ctx.remove("error");
                    ctx.remove("password");
                    ctx.insert("username".to_string(), json!(username));
                } else {
                    session.context = json!({ "username": username });
                }

                Ok(NodeOutcome::Continue {
                    output: "success".to_string(),
                })
            }
            Err(Error::UserAlreadyExists) => {
                self.reject_registration(session, username, "User already exists")
                    .await
            }
            Err(err) => Err(err),
        }
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            node = "registration_authenticator",
            phase = "on_exit"
        )
    )]
    async fn on_exit(&self, session: &mut AuthenticationSession) -> Result<()> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.remove("password");
        }
        Ok(())
    }
}

impl RegistrationAuthenticator {
    async fn reject_registration(
        &self,
        session: &mut AuthenticationSession,
        username: &str,
        reason: &str,
    ) -> Result<NodeOutcome> {
        if let Some(ctx) = session.context.as_object_mut() {
            ctx.insert("error".to_string(), json!(reason));
            ctx.insert("username".to_string(), json!(username));
        } else {
            session.context = json!({
                "error": reason,
                "username": username,
            });
        }

        Ok(NodeOutcome::Reject {
            error: reason.to_string(),
        })
    }
}
