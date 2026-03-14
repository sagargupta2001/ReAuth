use crate::adapters::persistence::connection::Database;
use crate::domain::recovery_attempt::RecoveryAttempt;
use crate::error::{Error, Result};
use crate::ports::recovery_attempt_repository::RecoveryAttemptRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteRecoveryAttemptRepository {
    pool: Database,
}

impl SqliteRecoveryAttemptRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RecoveryAttemptRecord {
    realm_id: String,
    identifier: String,
    window_started_at: DateTime<Utc>,
    attempt_count: i64,
    updated_at: DateTime<Utc>,
}

impl RecoveryAttemptRecord {
    fn into_attempt(self) -> Result<RecoveryAttempt> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in recovery attempts".to_string()))?;
        Ok(RecoveryAttempt {
            realm_id,
            identifier: self.identifier,
            window_started_at: self.window_started_at,
            attempt_count: self.attempt_count,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl RecoveryAttemptRepository for SqliteRecoveryAttemptRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "recovery_attempts", db_op = "select")
    )]
    async fn find(&self, realm_id: &Uuid, identifier: &str) -> Result<Option<RecoveryAttempt>> {
        let record: Option<RecoveryAttemptRecord> =
            sqlx::query_as("SELECT * FROM recovery_attempts WHERE realm_id = ? AND identifier = ?")
                .bind(realm_id.to_string())
                .bind(identifier)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        record.map(RecoveryAttemptRecord::into_attempt).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "recovery_attempts", db_op = "upsert")
    )]
    async fn upsert(&self, attempt: &RecoveryAttempt) -> Result<()> {
        sqlx::query(
            "INSERT INTO recovery_attempts (
                realm_id, identifier, window_started_at, attempt_count, updated_at
            ) VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(realm_id, identifier) DO UPDATE SET
                window_started_at = excluded.window_started_at,
                attempt_count = excluded.attempt_count,
                updated_at = excluded.updated_at",
        )
        .bind(attempt.realm_id.to_string())
        .bind(&attempt.identifier)
        .bind(attempt.window_started_at)
        .bind(attempt.attempt_count)
        .bind(attempt.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
