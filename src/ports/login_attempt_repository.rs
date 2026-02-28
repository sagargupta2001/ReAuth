use crate::domain::login_attempt::LoginAttempt;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LoginAttemptRepository: Send + Sync {
    async fn find(&self, realm_id: &Uuid, username: &str) -> Result<Option<LoginAttempt>>;
    async fn record_failure(
        &self,
        realm_id: &Uuid,
        username: &str,
        threshold: i64,
        lockout_duration_secs: i64,
    ) -> Result<LoginAttempt>;
    async fn clear(&self, realm_id: &Uuid, username: &str) -> Result<()>;
}
