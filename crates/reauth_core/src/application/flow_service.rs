use crate::{domain::auth_flow::AuthFlow, error::Result, ports::flow_repository::FlowRepository};
use std::sync::Arc;
use uuid::Uuid;

pub struct FlowService {
    flow_repo: Arc<dyn FlowRepository>,
}

impl FlowService {
    pub fn new(flow_repo: Arc<dyn FlowRepository>) -> Self {
        Self { flow_repo }
    }

    pub async fn list_flows(&self, realm_id: Uuid) -> Result<Vec<AuthFlow>> {
        self.flow_repo.list_flows_by_realm(&realm_id).await
    }
}
