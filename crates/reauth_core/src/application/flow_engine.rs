use crate::domain::user::User;
use crate::ports::user_repository::UserRepository;
use crate::{
    application::auth_service::AuthService,
    constants::DEFAULT_REALM_NAME,
    domain::auth_flow::{AuthContext, AuthStepResult, LoginSession},
    error::{Error, Result},
    ports::{
        authenticator::Authenticator, flow_repository::FlowRepository,
        realm_repository::RealmRepository,
    },
};
use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Holds a registry of all *available* authenticator implementations.
pub struct AuthenticatorRegistry {
    implementations: HashMap<String, Arc<dyn Authenticator>>,
}
impl AuthenticatorRegistry {
    pub fn new(implementations: HashMap<String, Arc<dyn Authenticator>>) -> Self {
        Self { implementations }
    }
    pub fn get(&self, name: &str) -> Option<Arc<dyn Authenticator>> {
        self.implementations.get(name).cloned()
    }
}

/// The main application service for orchestrating login flows.
pub struct FlowEngine {
    registry: Arc<AuthenticatorRegistry>,
    flow_repo: Arc<dyn FlowRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    auth_service: Arc<AuthService>,
    user_repo: Arc<dyn UserRepository>,
}

impl FlowEngine {
    pub fn new(
        registry: Arc<AuthenticatorRegistry>,
        flow_repo: Arc<dyn FlowRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        auth_service: Arc<AuthService>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            registry,
            flow_repo,
            realm_repo,
            auth_service,
            user_repo,
        }
    }

    /// Starts a new login flow (e.g., "browser-login").
    pub async fn start_login_flow(&self, realm_id: Uuid) -> Result<(LoginSession, AuthStepResult)> {
        // Find the flow for this realm
        let flow = self
            .flow_repo
            .find_flow_by_name(&realm_id, "browser-login")
            .await?
            .ok_or(Error::FlowNotFound("browser-login".to_string()))?;

        // Create and save a new login session
        let login_session = LoginSession {
            id: Uuid::new_v4(),
            realm_id,
            flow_id: flow.id,
            current_step: 0,
            user_id: None,
            state_data: None,
            expires_at: Utc::now() + Duration::seconds(600), // 10-minute login flow
        };
        self.flow_repo.save_login_session(&login_session).await?;

        // Get the first step and call its `challenge` method
        let challenge = self.challenge_current_step(&login_session).await?;
        Ok((login_session, challenge))
    }

    /// Processes a user's submission for the current step in their login flow.
    pub async fn process_login_step(
        &self,
        session_id: Uuid,
        credentials: HashMap<String, String>,
    ) -> Result<(Option<LoginSession>, AuthStepResult, Option<User>)> {
        let mut login_session = self
            .flow_repo
            .find_login_session_by_id(&session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;

        let steps = self
            .flow_repo
            .find_steps_for_flow(&login_session.flow_id)
            .await?;
        let current_step = steps
            .get(login_session.current_step as usize)
            .ok_or(Error::InvalidLoginStep)?;

        let authenticator = self.registry.get(&current_step.authenticator_name).ok_or(
            Error::AuthenticatorNotFound(current_step.authenticator_name.clone()),
        )?;

        let config = self
            .flow_repo
            .find_config_for_authenticator(
                &login_session.realm_id,
                &current_step.authenticator_name,
            )
            .await?;

        let mut context = AuthContext {
            realm_id: login_session.realm_id,
            login_session: login_session.clone(),
            config,
            credentials,
        };

        match authenticator.execute(&mut context).await? {
            AuthStepResult::Success => {
                // Update the login session with new state (e.g., the user_id)
                self.flow_repo
                    .update_login_session(&context.login_session)
                    .await?;

                // --- Flow is 100% complete ---
                if (context.login_session.current_step as usize) == steps.len() - 1 {
                    // 1. Capture the session data before deletion
                    let final_session = context.login_session.clone();

                    // 2. Delete from DB
                    self.flow_repo
                        .delete_login_session(&final_session.id)
                        .await?;

                    // 3. Get User
                    let user_id = final_session.user_id.ok_or(Error::InvalidLoginStep)?;
                    let user = self
                        .user_repo
                        .find_by_id(&user_id)
                        .await?
                        .ok_or(Error::UserNotFound)?;

                    // 4. Return `Some(final_session)` so the handler can read OIDC data
                    Ok((Some(final_session), AuthStepResult::Success, Some(user)))
                } else {
                    // --- Advance to the next step ---
                    login_session.current_step += 1;
                    self.flow_repo.update_login_session(&login_session).await?;
                    let challenge = self.challenge_current_step(&login_session).await?;
                    Ok((Some(login_session), challenge, None))
                }
            }
            result @ (AuthStepResult::Failure { .. }
            | AuthStepResult::Challenge { .. }
            | AuthStepResult::Redirect { .. }) => Ok((Some(login_session), result, None)),
        }
    }

    /// Helper to find the current step and call its `challenge` method.
    async fn challenge_current_step(&self, session: &LoginSession) -> Result<AuthStepResult> {
        let steps = self.flow_repo.find_steps_for_flow(&session.flow_id).await?;
        let current_step = steps
            .get(session.current_step as usize)
            .ok_or(Error::InvalidLoginStep)?;

        let authenticator = self.registry.get(&current_step.authenticator_name).ok_or(
            Error::AuthenticatorNotFound(current_step.authenticator_name.clone()),
        )?;

        let config = self
            .flow_repo
            .find_config_for_authenticator(&session.realm_id, &current_step.authenticator_name)
            .await?;

        let context = AuthContext {
            realm_id: session.realm_id,
            login_session: session.clone(),
            config,
            credentials: HashMap::new(), // No credentials for a challenge
        };

        authenticator.challenge(&context).await
    }

    pub async fn update_login_session(&self, session: &LoginSession) -> Result<()> {
        self.flow_repo.update_login_session(session).await
    }
}
