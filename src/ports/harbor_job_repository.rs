use crate::domain::harbor_job::HarborJob;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait HarborJobRepository: Send + Sync {
    async fn create(&self, job: &HarborJob) -> Result<()>;
    async fn update_progress(
        &self,
        job_id: &Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) -> Result<()>;
    async fn update_total(&self, job_id: &Uuid, total: i64) -> Result<()>;
    async fn update_artifact(
        &self,
        job_id: &Uuid,
        path: &str,
        filename: &str,
        content_type: &str,
    ) -> Result<()>;
    async fn mark_completed(
        &self,
        job_id: &Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) -> Result<()>;
    async fn mark_failed(&self, job_id: &Uuid, error_message: &str) -> Result<()>;
    async fn list_recent(&self, realm_id: &Uuid, limit: i64) -> Result<Vec<HarborJob>>;
    async fn find_by_id(&self, job_id: &Uuid) -> Result<Option<HarborJob>>;
}
