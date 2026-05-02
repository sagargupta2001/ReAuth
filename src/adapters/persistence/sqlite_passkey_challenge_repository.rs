use crate::adapters::persistence::connection::Database;
use crate::domain::passkey_challenge::{PasskeyChallenge, PasskeyChallengeKind};
use crate::error::{Error, Result};
use crate::ports::passkey_challenge_repository::PasskeyChallengeRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use tracing::instrument;
use uuid::Uuid;

pub struct SqlitePasskeyChallengeRepository {
    pool: Database,
}

impl SqlitePasskeyChallengeRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PasskeyChallengeRecord {
    id: String,
    realm_id: String,
    auth_session_id: String,
    user_id: Option<String>,
    challenge_kind: String,
    challenge_hash: String,
    rp_id: String,
    allowed_origins_json: String,
    expires_at: DateTime<Utc>,
    consumed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl PasskeyChallengeRecord {
    fn into_domain(self) -> Result<PasskeyChallenge> {
        let kind = match self.challenge_kind.as_str() {
            "authentication" => PasskeyChallengeKind::Authentication,
            "enrollment" => PasskeyChallengeKind::Enrollment,
            "reauthentication" => PasskeyChallengeKind::Reauthentication,
            other => {
                return Err(Error::System(format!(
                    "Invalid passkey challenge kind: {}",
                    other
                )));
            }
        };

        Ok(PasskeyChallenge {
            id: Uuid::parse_str(&self.id)
                .map_err(|_| Error::System("Invalid passkey challenge id".to_string()))?,
            realm_id: Uuid::parse_str(&self.realm_id)
                .map_err(|_| Error::System("Invalid passkey realm id".to_string()))?,
            auth_session_id: Uuid::parse_str(&self.auth_session_id)
                .map_err(|_| Error::System("Invalid auth session id".to_string()))?,
            user_id: self.user_id.and_then(|value| Uuid::parse_str(&value).ok()),
            challenge_kind: kind,
            challenge_hash: self.challenge_hash,
            rp_id: self.rp_id,
            allowed_origins_json: self.allowed_origins_json,
            expires_at: self.expires_at,
            consumed_at: self.consumed_at,
            created_at: self.created_at,
        })
    }
}

#[async_trait]
impl PasskeyChallengeRepository for SqlitePasskeyChallengeRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "insert")
    )]
    async fn create(&self, challenge: &PasskeyChallenge) -> Result<()> {
        sqlx::query(
            "INSERT INTO passkey_challenges (
                id, realm_id, auth_session_id, user_id, challenge_kind, challenge_hash,
                rp_id, allowed_origins_json, expires_at, consumed_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(challenge.id.to_string())
        .bind(challenge.realm_id.to_string())
        .bind(challenge.auth_session_id.to_string())
        .bind(challenge.user_id.map(|value| value.to_string()))
        .bind(challenge.challenge_kind.as_str())
        .bind(&challenge.challenge_hash)
        .bind(&challenge.rp_id)
        .bind(&challenge.allowed_origins_json)
        .bind(challenge.expires_at)
        .bind(challenge.consumed_at)
        .bind(challenge.created_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "select")
    )]
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<PasskeyChallenge>> {
        let record: Option<PasskeyChallengeRecord> =
            sqlx::query_as("SELECT * FROM passkey_challenges WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        record.map(PasskeyChallengeRecord::into_domain).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "consume")
    )]
    async fn consume_if_active(
        &self,
        id: &Uuid,
        realm_id: &Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<PasskeyChallenge>> {
        let result = sqlx::query(
            "UPDATE passkey_challenges
             SET consumed_at = ?
             WHERE id = ? AND realm_id = ? AND consumed_at IS NULL AND expires_at > ?",
        )
        .bind(now)
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(now)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        if result.rows_affected() != 1 {
            return Ok(None);
        }

        self.find_by_id(id).await
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "delete")
    )]
    async fn delete_expired_before(&self, cutoff: DateTime<Utc>, batch_size: i64) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM passkey_challenges
             WHERE id IN (
                SELECT id
                FROM passkey_challenges
                WHERE expires_at < ? OR consumed_at IS NOT NULL
                LIMIT ?
             )",
        )
        .bind(cutoff)
        .bind(batch_size.max(1))
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "count")
    )]
    async fn count_unconsumed(&self, realm_id: &Uuid) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS count
             FROM passkey_challenges
             WHERE realm_id = ? AND consumed_at IS NULL",
        )
        .bind(realm_id.to_string())
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row.get::<i64, _>("count").max(0) as u64)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "passkey_challenges", db_op = "count")
    )]
    async fn count_expired_unconsumed(&self, realm_id: &Uuid, now: DateTime<Utc>) -> Result<u64> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS count
             FROM passkey_challenges
             WHERE realm_id = ? AND consumed_at IS NULL AND expires_at <= ?",
        )
        .bind(realm_id.to_string())
        .bind(now)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(row.get::<i64, _>("count").max(0) as u64)
    }
}
