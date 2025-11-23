use crate::{domain::realm::Realm, error::Result};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RealmRepository: Send + Sync {
    async fn create(&self, realm: &Realm) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Realm>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Realm>>;
    async fn list_all(&self) -> Result<Vec<Realm>>;
}
