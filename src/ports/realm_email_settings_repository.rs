use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::realm_email_settings::RealmEmailSettings;
use crate::error::Result;

#[async_trait]
pub trait RealmEmailSettingsRepository: Send + Sync {
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmEmailSettings>>;
    async fn upsert(&self, settings: &RealmEmailSettings) -> Result<()>;
}
