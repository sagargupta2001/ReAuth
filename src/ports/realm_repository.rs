use crate::domain::auth_flow::AuthFlow;
use crate::ports::transaction_manager::Transaction;
use crate::{domain::realm::Realm, error::Result};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RealmRepository: Send + Sync {
    async fn create<'a>(&self, realm: &Realm, tx: Option<&'a mut dyn Transaction>) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Realm>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Realm>>;
    async fn list_all(&self) -> Result<Vec<Realm>>;
    async fn update<'a>(&self, realm: &Realm, tx: Option<&'a mut dyn Transaction>) -> Result<()>;
    async fn list_flows_by_realm(&self, realm_id: &Uuid) -> Result<Vec<AuthFlow>>;
    async fn update_flow_binding<'a>(
        &self,
        realm_id: &Uuid,
        slot: &str,
        flow_id: &Uuid,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()>;
}
