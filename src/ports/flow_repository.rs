use crate::ports::transaction_manager::Transaction;
use crate::{domain::auth_flow::AuthFlow, error::Result};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FlowRepository: Send + Sync {
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>>;
    async fn find_flow_by_id(&self, flow_id: &Uuid) -> Result<Option<AuthFlow>>;
    async fn create_flow<'a>(
        &self,
        flow: &AuthFlow,
        tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()>;
    async fn list_flows_by_realm(&self, realm_id: &Uuid) -> Result<Vec<AuthFlow>>;

    /// Permanently removes a runtime flow. Versions and deployments referencing
    /// it are cascaded by the schema's foreign keys. Defaults to a no-op so test
    /// doubles need not implement it.
    async fn delete_flow(&self, _flow_id: &Uuid) -> Result<()> {
        Ok(())
    }
}
