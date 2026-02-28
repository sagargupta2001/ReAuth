use crate::adapters::persistence::connection::Database;
use crate::domain::login_attempt::LoginAttempt;
use crate::error::{Error, Result};
use crate::ports::login_attempt_repository::LoginAttemptRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteLoginAttemptRepository {
    pool: Database,
}

impl SqliteLoginAttemptRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LoginAttemptRepository for SqliteLoginAttemptRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "login_attempts", db_op = "select")
    )]
    async fn find(&self, realm_id: &Uuid, username: &str) -> Result<Option<LoginAttempt>> {
        let attempt =
            sqlx::query_as("SELECT * FROM login_attempts WHERE realm_id = ? AND username = ?")
                .bind(realm_id.to_string())
                .bind(username)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(attempt)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "login_attempts", db_op = "upsert")
    )]
    async fn record_failure(
        &self,
        realm_id: &Uuid,
        username: &str,
        threshold: i64,
        lockout_duration_secs: i64,
    ) -> Result<LoginAttempt> {
        let now = Utc::now();
        let locked_until = now + Duration::seconds(lockout_duration_secs);

        let attempt = sqlx::query_as(
            r#"
INSERT INTO login_attempts (
    realm_id,
    username,
    failed_count,
    last_failed_at,
    locked_until,
    created_at,
    updated_at
) VALUES (
    ?, ?, 1, ?,
    CASE WHEN ? <= 1 THEN ? ELSE NULL END,
    ?, ?
)
ON CONFLICT(realm_id, username) DO UPDATE SET
    failed_count = failed_count + 1,
    last_failed_at = excluded.last_failed_at,
    locked_until = CASE
        WHEN failed_count + 1 >= ? THEN excluded.locked_until
        ELSE locked_until
    END,
    updated_at = excluded.updated_at
RETURNING *
"#,
        )
        .bind(realm_id.to_string())
        .bind(username)
        .bind(now)
        .bind(threshold)
        .bind(locked_until)
        .bind(now)
        .bind(now)
        .bind(threshold)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(attempt)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "login_attempts", db_op = "delete")
    )]
    async fn clear(&self, realm_id: &Uuid, username: &str) -> Result<()> {
        sqlx::query("DELETE FROM login_attempts WHERE realm_id = ? AND username = ?")
            .bind(realm_id.to_string())
            .bind(username)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}
