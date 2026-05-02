use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::passkey_challenge::PasskeyChallenge;
use crate::error::Result;

#[async_trait]
pub trait PasskeyChallengeRepository: Send + Sync {
    async fn create(&self, challenge: &PasskeyChallenge) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<PasskeyChallenge>>;
    async fn consume_if_active(
        &self,
        id: &Uuid,
        realm_id: &Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<PasskeyChallenge>>;
    async fn delete_expired_before(&self, cutoff: DateTime<Utc>, batch_size: i64) -> Result<u64>;
    async fn count_unconsumed(&self, realm_id: &Uuid) -> Result<u64>;
    async fn count_expired_unconsumed(&self, realm_id: &Uuid, now: DateTime<Utc>) -> Result<u64>;
}
