use crate::adapters::persistence::connection::Database;
use crate::domain::auth_session_action::AuthSessionAction;
use crate::error::{Error, Result};
use crate::ports::auth_session_action_repository::AuthSessionActionRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteAuthSessionActionRepository {
    pool: Database,
}

impl SqliteAuthSessionActionRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthSessionActionRepository for SqliteAuthSessionActionRepository {
    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "auth_session_actions",
            db_op = "insert"
        )
    )]
    async fn create(&self, action: &AuthSessionAction) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO auth_session_actions (
    id,
    session_id,
    realm_id,
    action_type,
    token_hash,
    payload_json,
    resume_node_id,
    expires_at,
    consumed_at,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(action.id.to_string())
        .bind(action.session_id.to_string())
        .bind(action.realm_id.to_string())
        .bind(&action.action_type)
        .bind(&action.token_hash)
        .bind(sqlx::types::Json(&action.payload))
        .bind(&action.resume_node_id)
        .bind(action.expires_at)
        .bind(action.consumed_at)
        .bind(action.created_at)
        .bind(action.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "auth_session_actions",
            db_op = "select"
        )
    )]
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<AuthSessionAction>> {
        let action = sqlx::query_as(
            r#"
SELECT
    id,
    session_id,
    realm_id,
    action_type,
    token_hash,
    COALESCE(payload_json, '{}') AS payload_json,
    resume_node_id,
    expires_at,
    consumed_at,
    created_at,
    updated_at
FROM auth_session_actions
WHERE token_hash = ?
"#,
        )
        .bind(token_hash)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(action)
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "auth_session_actions",
            db_op = "update"
        )
    )]
    async fn mark_consumed(&self, id: &Uuid) -> Result<()> {
        sqlx::query("UPDATE auth_session_actions SET consumed_at = ?, updated_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(Utc::now())
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "auth_session_actions",
            db_op = "delete"
        )
    )]
    async fn delete_expired_before(&self, cutoff: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM auth_session_actions WHERE expires_at < ? OR consumed_at IS NOT NULL",
        )
        .bind(cutoff)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(result.rows_affected())
    }
}
