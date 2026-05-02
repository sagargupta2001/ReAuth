use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::error::Result;

#[async_trait]
pub trait RealmPasskeySettingsRepository: Send + Sync {
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmPasskeySettings>>;
    async fn upsert(&self, settings: &RealmPasskeySettings) -> Result<()>;
}
