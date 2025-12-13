use crate::{
    domain::{
        flow::{FlowDeployment, FlowDraft, FlowVersion},
        pagination::{PageRequest, PageResponse},
    },
    error::Result,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FlowStore: Send + Sync {
    // --- Drafts ---
    async fn create_draft(&self, draft: &FlowDraft) -> Result<()>;
    async fn update_draft(&self, draft: &FlowDraft) -> Result<()>;
    async fn get_draft_by_id(&self, id: &Uuid) -> Result<Option<FlowDraft>>;
    async fn list_drafts(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowDraft>>;

    async fn list_all_drafts(&self, realm_id: &Uuid) -> Result<Vec<FlowDraft>>;
    async fn delete_draft(&self, id: &Uuid) -> Result<()>;

    // --- Versions ---
    async fn create_version(&self, version: &FlowVersion) -> Result<()>;
    async fn get_version(&self, id: &Uuid) -> Result<Option<FlowVersion>>;
    async fn list_versions(
        &self,
        flow_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowVersion>>;

    // --- Deployments ---
    async fn set_deployment(&self, deployment: &FlowDeployment) -> Result<()>;
    async fn get_deployment(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
    ) -> Result<Option<FlowDeployment>>;
    async fn get_latest_version_number(&self, flow_id: &Uuid) -> Result<Option<i32>>;
    async fn get_latest_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>>;
    async fn get_deployed_version_number(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
        flow_id: &Uuid,
    ) -> Result<Option<i32>>;

    async fn get_version_by_number(
        &self,
        flow_id: &Uuid,
        version_number: i32,
    ) -> Result<Option<FlowVersion>>;

    async fn get_active_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>>;
}
