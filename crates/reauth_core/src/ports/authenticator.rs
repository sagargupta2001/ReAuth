use crate::{
    domain::auth_flow::{AuthContext, AuthStepResult},
    error::Result,
};
use async_trait::async_trait;

/// The "Port" that all authenticators must implement.
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// A unique name for this authenticator, e.g., "builtin-password-auth"
    fn name(&self) -> &'static str;

    /// Called to get the UI for this step.
    /// Returns a Challenge result to tell the frontend what to render.
    async fn challenge(&self, context: &AuthContext) -> Result<AuthStepResult>;

    /// Called when the user submits the UI for this step.
    /// Processes the credentials and returns a new result.
    async fn execute(&self, context: &mut AuthContext) -> Result<AuthStepResult>;
}
