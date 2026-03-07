use crate::adapters::persistence::connection::Database;
use crate::domain::harbor_job::HarborJob;
use crate::error::{Error, Result};
use crate::ports::harbor_job_repository::HarborJobRepository;
use async_trait::async_trait;
use uuid::Uuid;

pub struct SqliteHarborJobRepository {
    pool: Database,
}

impl SqliteHarborJobRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HarborJobRepository for SqliteHarborJobRepository {
    async fn create(&self, job: &HarborJob) -> Result<()> {
        sqlx::query(
            "INSERT INTO harbor_jobs (
                id, realm_id, job_type, status, scope, total_resources, processed_resources,
                created_count, updated_count, dry_run, conflict_policy, artifact_path, artifact_filename, artifact_content_type, error_message,
                created_at, updated_at, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(job.id.to_string())
        .bind(job.realm_id.to_string())
        .bind(&job.job_type)
        .bind(&job.status)
        .bind(&job.scope)
        .bind(job.total_resources)
        .bind(job.processed_resources)
        .bind(job.created_count)
        .bind(job.updated_count)
        .bind(job.dry_run)
        .bind(&job.conflict_policy)
        .bind(&job.artifact_path)
        .bind(&job.artifact_filename)
        .bind(&job.artifact_content_type)
        .bind(&job.error_message)
        .bind(job.created_at)
        .bind(job.updated_at)
        .bind(job.completed_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn update_progress(
        &self,
        job_id: &Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE harbor_jobs
             SET processed_resources = ?, created_count = ?, updated_count = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(processed)
        .bind(created)
        .bind(updated)
        .bind(job_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn update_total(&self, job_id: &Uuid, total: i64) -> Result<()> {
        sqlx::query(
            "UPDATE harbor_jobs
             SET total_resources = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(total)
        .bind(job_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn update_artifact(
        &self,
        job_id: &Uuid,
        path: &str,
        filename: &str,
        content_type: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE harbor_jobs
             SET artifact_path = ?, artifact_filename = ?, artifact_content_type = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(path)
        .bind(filename)
        .bind(content_type)
        .bind(job_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn mark_completed(
        &self,
        job_id: &Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE harbor_jobs
             SET status = 'completed', processed_resources = ?, created_count = ?, updated_count = ?,
                 updated_at = CURRENT_TIMESTAMP, completed_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(processed)
        .bind(created)
        .bind(updated)
        .bind(job_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn mark_failed(&self, job_id: &Uuid, error_message: &str) -> Result<()> {
        sqlx::query(
            "UPDATE harbor_jobs
             SET status = 'failed', error_message = ?, updated_at = CURRENT_TIMESTAMP, completed_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(error_message)
        .bind(job_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn list_recent(&self, realm_id: &Uuid, limit: i64) -> Result<Vec<HarborJob>> {
        let jobs = sqlx::query_as::<_, HarborJob>(
            "SELECT id, realm_id, job_type, status, scope, total_resources, processed_resources,
                    created_count, updated_count, dry_run, conflict_policy, artifact_path, artifact_filename, artifact_content_type, error_message,
                    created_at, updated_at, completed_at
             FROM harbor_jobs
             WHERE realm_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(realm_id.to_string())
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(jobs)
    }

    async fn find_by_id(&self, job_id: &Uuid) -> Result<Option<HarborJob>> {
        let job = sqlx::query_as::<_, HarborJob>(
            "SELECT id, realm_id, job_type, status, scope, total_resources, processed_resources,
                    created_count, updated_count, dry_run, conflict_policy, artifact_path, artifact_filename, artifact_content_type, error_message,
                    created_at, updated_at, completed_at
             FROM harbor_jobs
             WHERE id = ?",
        )
        .bind(job_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(job)
    }
}
