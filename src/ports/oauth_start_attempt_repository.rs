use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::oauth_start_attempt::OAuthStartAttempt;
use crate::error::Result;

#[async_trait]
pub trait OAuthStartAttemptRepository: Send + Sync {
    async fn find(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
        ip_address: &str,
    ) -> Result<Option<OAuthStartAttempt>>;
    async fn upsert(&self, attempt: &OAuthStartAttempt) -> Result<()>;
}
