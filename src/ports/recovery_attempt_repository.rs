use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::recovery_attempt::RecoveryAttempt;
use crate::error::Result;

#[async_trait]
pub trait RecoveryAttemptRepository: Send + Sync {
    async fn find(&self, realm_id: &Uuid, identifier: &str) -> Result<Option<RecoveryAttempt>>;
    async fn upsert(&self, attempt: &RecoveryAttempt) -> Result<()>;
}
