use crate::domain::realm_security_headers::RealmSecurityHeaders;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RealmSecurityHeadersRepository: Send + Sync {
    async fn find_by_realm_id(&self, realm_id: &Uuid) -> Result<Option<RealmSecurityHeaders>>;
    async fn upsert(&self, settings: &RealmSecurityHeaders) -> Result<()>;
}
