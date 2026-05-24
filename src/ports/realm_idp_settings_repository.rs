use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::realm_idp_settings::RealmIdpSettings;
use crate::error::Result;

#[async_trait]
pub trait RealmIdpSettingsRepository: Send + Sync {
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmIdpSettings>>;
    async fn upsert(&self, settings: &RealmIdpSettings) -> Result<()>;
}
