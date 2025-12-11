use crate::domain::auth_session::AuthenticationSession;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AuthSessionRepository: Send + Sync {
    // Create the session when user hits "Login"
    async fn create(&self, session: &AuthenticationSession) -> Result<()>;

    // Load state to process the next step
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<AuthenticationSession>>;

    // Save the new Program Counter and Context after a step runs
    async fn update(&self, session: &AuthenticationSession) -> Result<()>;

    // Cleanup (e.g. user closes tab or finishes flow)
    async fn delete(&self, id: &Uuid) -> Result<()>;
}
