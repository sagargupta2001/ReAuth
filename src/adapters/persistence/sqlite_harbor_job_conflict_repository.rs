use crate::adapters::persistence::connection::Database;
use crate::domain::harbor_job_conflict::HarborJobConflict;
use crate::error::{Error, Result};
use crate::ports::harbor_job_conflict_repository::HarborJobConflictRepository;
use async_trait::async_trait;
use uuid::Uuid;

pub struct SqliteHarborJobConflictRepository {
    pool: Database,
}

impl SqliteHarborJobConflictRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HarborJobConflictRepository for SqliteHarborJobConflictRepository {
    async fn create(&self, conflict: &HarborJobConflict) -> Result<()> {
        sqlx::query(
            "INSERT INTO harbor_job_conflicts (
                id, job_id, resource_key, action, policy, original_id, resolved_id, message, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(conflict.id.to_string())
        .bind(conflict.job_id.to_string())
        .bind(&conflict.resource_key)
        .bind(&conflict.action)
        .bind(&conflict.policy)
        .bind(&conflict.original_id)
        .bind(&conflict.resolved_id)
        .bind(&conflict.message)
        .bind(conflict.created_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn list_by_job(&self, job_id: &Uuid) -> Result<Vec<HarborJobConflict>> {
        let rows = sqlx::query_as::<_, HarborJobConflict>(
            "SELECT id, job_id, resource_key, action, policy, original_id, resolved_id, message, created_at
             FROM harbor_job_conflicts
             WHERE job_id = ?
             ORDER BY created_at ASC",
        )
        .bind(job_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows)
    }
}
