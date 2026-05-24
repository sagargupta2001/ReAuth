use crate::domain::identity_provider::IdentityProvider;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IdentityProviderRepository: Send + Sync {
    async fn create(&self, provider: &IdentityProvider) -> Result<()>;
    async fn update(&self, provider: &IdentityProvider) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<IdentityProvider>>;
    async fn find_by_alias(&self, realm_id: &Uuid, alias: &str)
        -> Result<Option<IdentityProvider>>;
    async fn list_by_realm(&self, realm_id: &Uuid) -> Result<Vec<IdentityProvider>>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
}
