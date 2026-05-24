use crate::domain::identity_provider::FederatedIdentity;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FederatedIdentityRepository: Send + Sync {
    async fn create(&self, identity: &FederatedIdentity) -> Result<()>;
    async fn update(&self, identity: &FederatedIdentity) -> Result<()>;
    async fn find_by_provider_subject(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
        subject: &str,
    ) -> Result<Option<FederatedIdentity>>;
    async fn list_by_user(&self, realm_id: &Uuid, user_id: &Uuid)
        -> Result<Vec<FederatedIdentity>>;
    async fn list_by_provider(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
    ) -> Result<Vec<FederatedIdentity>>;
    async fn count_by_provider(&self, realm_id: &Uuid, provider_id: &Uuid) -> Result<u64>;
    async fn delete_by_provider(&self, realm_id: &Uuid, provider_id: &Uuid) -> Result<u64>;
    async fn delete_by_id_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        federated_identity_id: &Uuid,
    ) -> Result<bool>;
}
