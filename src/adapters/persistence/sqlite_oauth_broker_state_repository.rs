use crate::adapters::persistence::connection::Database;
use crate::domain::identity_provider::OAuthBrokerState;
use crate::error::{Error, Result};
use crate::ports::oauth_broker_state_repository::OAuthBrokerStateRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteOAuthBrokerStateRepository {
    pool: Database,
}

impl SqliteOAuthBrokerStateRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OAuthBrokerStateRow {
    id: String,
    realm_id: String,
    provider_id: String,
    auth_session_id: String,
    pkce_verifier_hash: String,
    redirect_uri: String,
    nonce: Option<String>,
    expires_at: DateTime<Utc>,
    consumed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<OAuthBrokerStateRow> for OAuthBrokerState {
    type Error = Error;

    fn try_from(row: OAuthBrokerStateRow) -> Result<Self> {
        Ok(Self {
            id: Uuid::parse_str(&row.id)
                .map_err(|_| Error::System("Invalid broker state id".into()))?,
            realm_id: Uuid::parse_str(&row.realm_id)
                .map_err(|_| Error::System("Invalid broker state realm id".into()))?,
            provider_id: Uuid::parse_str(&row.provider_id)
                .map_err(|_| Error::System("Invalid broker state provider id".into()))?,
            auth_session_id: Uuid::parse_str(&row.auth_session_id)
                .map_err(|_| Error::System("Invalid broker state session id".into()))?,
            pkce_verifier_hash: row.pkce_verifier_hash,
            redirect_uri: row.redirect_uri,
            nonce: row.nonce,
            expires_at: row.expires_at,
            consumed_at: row.consumed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl OAuthBrokerStateRepository for SqliteOAuthBrokerStateRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "oauth_broker_states", db_op = "insert")
    )]
    async fn create(&self, state: &OAuthBrokerState) -> Result<()> {
        sqlx::query(
            "INSERT INTO oauth_broker_states (
                id, realm_id, provider_id, auth_session_id, pkce_verifier_hash, redirect_uri,
                nonce, expires_at, consumed_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(state.id.to_string())
        .bind(state.realm_id.to_string())
        .bind(state.provider_id.to_string())
        .bind(state.auth_session_id.to_string())
        .bind(&state.pkce_verifier_hash)
        .bind(&state.redirect_uri)
        .bind(&state.nonce)
        .bind(state.expires_at)
        .bind(state.consumed_at)
        .bind(state.created_at)
        .bind(state.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "oauth_broker_states", db_op = "select")
    )]
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<OAuthBrokerState>> {
        let row: Option<OAuthBrokerStateRow> =
            sqlx::query_as("SELECT * FROM oauth_broker_states WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        row.map(TryInto::try_into).transpose()
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "oauth_broker_states", db_op = "update")
    )]
    async fn mark_consumed_if_active(&self, id: &Uuid, now: DateTime<Utc>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE oauth_broker_states
             SET consumed_at = ?, updated_at = ?
             WHERE id = ? AND consumed_at IS NULL AND expires_at > ?",
        )
        .bind(now)
        .bind(now)
        .bind(id.to_string())
        .bind(now)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected() == 1)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "oauth_broker_states", db_op = "delete")
    )]
    async fn delete_expired_before(&self, cutoff: DateTime<Utc>, batch_size: i64) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM oauth_broker_states
             WHERE id IN (
                 SELECT id
                 FROM oauth_broker_states
                 WHERE expires_at < ? OR consumed_at IS NOT NULL
                 ORDER BY created_at ASC
                 LIMIT ?
             )",
        )
        .bind(cutoff)
        .bind(batch_size)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected())
    }
}
