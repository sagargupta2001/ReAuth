use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::error::Result;

#[async_trait]
pub trait RealmRecoverySettingsRepository: Send + Sync {
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmRecoverySettings>>;
    async fn upsert(&self, settings: &RealmRecoverySettings) -> Result<()>;
}
