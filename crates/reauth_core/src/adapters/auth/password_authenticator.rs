use crate::{
    domain::{
        auth_flow::{AuthContext, AuthStepResult},
        crypto::HashedPassword,
    },
    error::{Error, Result},
    ports::{authenticator::Authenticator, user_repository::UserRepository},
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::warn;

/// A built-in, in-process authenticator that checks the user's password.
pub struct PasswordAuthenticator {
    user_repo: Arc<dyn UserRepository>,
}

impl PasswordAuthenticator {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

#[async_trait]
impl Authenticator for PasswordAuthenticator {
    fn name(&self) -> &'static str {
        "builtin-password-auth"
    }

    /// For the password form, the "challenge" is to show the login page.
    async fn challenge(&self, _context: &AuthContext) -> Result<AuthStepResult> {
        Ok(AuthStepResult::Challenge {
            challenge_name: "FORM".to_string(),
            challenge_page: "/login".to_string(), // Tells the UI which route to render
        })
    }

    /// Executes the password check.
    async fn execute(&self, context: &mut AuthContext) -> Result<AuthStepResult> {
        let username = context
            .credentials
            .get("username")
            .ok_or(Error::InvalidCredentials)?;
        let password = context
            .credentials
            .get("password")
            .ok_or(Error::InvalidCredentials)?;

        let user = match self.user_repo.find_by_username(username).await? {
            Some(user) => user,
            None => {
                warn!("Authentication failed: User not found '{}'", username);
                return Ok(AuthStepResult::Failure {
                    message: "Invalid credentials".to_string(),
                });
            }
        };

        let hashed_password = HashedPassword::from_hash(&user.hashed_password)?;
        if !hashed_password.verify(password)? {
            warn!(
                "Authentication failed: Invalid password for user '{}'",
                username
            );
            return Ok(AuthStepResult::Failure {
                message: "Invalid credentials".to_string(),
            });
        }

        // Success! Attach the authenticated user to the context
        // so subsequent steps (like MFA) can use it.
        context.login_session.user_id = Some(user.id);
        Ok(AuthStepResult::Success)
    }
}
