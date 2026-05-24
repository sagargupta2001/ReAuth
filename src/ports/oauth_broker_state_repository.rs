use crate::domain::identity_provider::OAuthBrokerState;
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait OAuthBrokerStateRepository: Send + Sync {
    async fn create(&self, state: &OAuthBrokerState) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<OAuthBrokerState>>;
    async fn mark_consumed_if_active(&self, id: &Uuid, now: DateTime<Utc>) -> Result<bool>;
    async fn delete_expired_before(&self, cutoff: DateTime<Utc>, batch_size: i64) -> Result<u64>;
}
