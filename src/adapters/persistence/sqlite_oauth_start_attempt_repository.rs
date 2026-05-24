use crate::adapters::persistence::connection::Database;
use crate::domain::oauth_start_attempt::OAuthStartAttempt;
use crate::error::{Error, Result};
use crate::ports::oauth_start_attempt_repository::OAuthStartAttemptRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteOAuthStartAttemptRepository {
    pool: Database,
}

impl SqliteOAuthStartAttemptRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OAuthStartAttemptRecord {
    realm_id: String,
    provider_id: String,
    ip_address: String,
    window_started_at: DateTime<Utc>,
    attempt_count: i64,
    updated_at: DateTime<Utc>,
}

impl OAuthStartAttemptRecord {
    fn into_attempt(self) -> Result<OAuthStartAttempt> {
        let realm_id = Uuid::parse_str(&self.realm_id)
            .map_err(|_| Error::System("Invalid realm id in oauth start attempts".to_string()))?;
        let provider_id = Uuid::parse_str(&self.provider_id).map_err(|_| {
            Error::System("Invalid provider id in oauth start attempts".to_string())
        })?;
        Ok(OAuthStartAttempt {
            realm_id,
            provider_id,
            ip_address: self.ip_address,
            window_started_at: self.window_started_at,
            attempt_count: self.attempt_count,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl OAuthStartAttemptRepository for SqliteOAuthStartAttemptRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "oauth_start_attempts",
            db_op = "select"
        )
    )]
    async fn find(
        &self,
        realm_id: &Uuid,
        provider_id: &Uuid,
        ip_address: &str,
    ) -> Result<Option<OAuthStartAttempt>> {
        let record: Option<OAuthStartAttemptRecord> = sqlx::query_as(
            "SELECT * FROM oauth_start_attempts WHERE realm_id = ? AND provider_id = ? AND ip_address = ?",
        )
        .bind(realm_id.to_string())
        .bind(provider_id.to_string())
        .bind(ip_address)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        record
            .map(OAuthStartAttemptRecord::into_attempt)
            .transpose()
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "oauth_start_attempts",
            db_op = "upsert"
        )
    )]
    async fn upsert(&self, attempt: &OAuthStartAttempt) -> Result<()> {
        sqlx::query(
            "INSERT INTO oauth_start_attempts (
                realm_id, provider_id, ip_address, window_started_at, attempt_count, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(realm_id, provider_id, ip_address) DO UPDATE SET
                window_started_at = excluded.window_started_at,
                attempt_count = excluded.attempt_count,
                updated_at = excluded.updated_at",
        )
        .bind(attempt.realm_id.to_string())
        .bind(attempt.provider_id.to_string())
        .bind(&attempt.ip_address)
        .bind(attempt.window_started_at)
        .bind(attempt.attempt_count)
        .bind(attempt.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }
}
