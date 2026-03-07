use crate::domain::harbor_job_conflict::HarborJobConflict;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait HarborJobConflictRepository: Send + Sync {
    async fn create(&self, conflict: &HarborJobConflict) -> Result<()>;
    async fn list_by_job(&self, job_id: &Uuid) -> Result<Vec<HarborJobConflict>>;
}
