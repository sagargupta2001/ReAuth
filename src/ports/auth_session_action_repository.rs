use crate::domain::auth_session_action::AuthSessionAction;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AuthSessionActionRepository: Send + Sync {
    async fn create(&self, action: &AuthSessionAction) -> Result<()>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<AuthSessionAction>>;
    async fn mark_consumed(&self, id: &Uuid) -> Result<()>;
    async fn delete_expired_before(&self, cutoff: chrono::DateTime<chrono::Utc>) -> Result<u64>;
}
