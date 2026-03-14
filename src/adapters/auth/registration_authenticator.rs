use crate::application::rbac_service::RbacService;
use crate::application::realm_policy::RealmCapabilities;
use crate::application::user_service::UserService;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::{Error, Result};
use crate::ports::realm_repository::RealmRepository;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{instrument, warn};

pub struct RegistrationAuthenticator {
    user_service: Arc<UserService>,
    realm_repo: Arc<dyn RealmRepository>,
    rbac_service: Arc<RbacService>,
}

impl RegistrationAuthenticator {
    pub fn new(
        user_service: Arc<UserService>,
        realm_repo: Arc<dyn RealmRepository>,
        rbac_service: Arc<RbacService>,
    ) -> Self {
        Self {
            user_service,
            realm_repo,
            rbac_service,
        }
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
        let realm = self
            .realm_repo
            .find_by_id(&session.realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(session.realm_id.to_string()))?;
        let capabilities = RealmCapabilities::from_realm(&realm);
        if !capabilities.registration_enabled {
            return self
                .reject_registration(session, "", "Registration is disabled")
                .await;
        }

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

                for role_id in capabilities.default_registration_role_ids {
                    if let Err(err) = self
                        .rbac_service
                        .assign_role_to_user(session.realm_id, user.id, role_id)
                        .await
                    {
                        warn!(
                            "Failed to assign default registration role {}: {}",
                            role_id, err
                        );
                    }
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
