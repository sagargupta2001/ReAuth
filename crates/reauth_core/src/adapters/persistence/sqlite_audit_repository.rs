use crate::adapters::persistence::connection::Database;
use crate::domain::audit::AuditEvent;
use crate::error::{Error, Result};
use crate::ports::audit_repository::AuditRepository;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

pub struct SqliteAuditRepository {
    pool: Database,
}

impl SqliteAuditRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct AuditEventRow {
    id: String,
    realm_id: String,
    actor_user_id: Option<String>,
    action: String,
    target_type: String,
    target_id: Option<String>,
    metadata: String,
    created_at: String,
}

#[async_trait]
impl AuditRepository for SqliteAuditRepository {
    async fn insert(&self, event: &AuditEvent) -> Result<()> {
        let metadata = serde_json::to_string(&event.metadata).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO audit_events (id, realm_id, actor_user_id, action, target_type, target_id, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.id.to_string())
        .bind(event.realm_id.to_string())
        .bind(event.actor_user_id.map(|id| id.to_string()))
        .bind(&event.action)
        .bind(&event.target_type)
        .bind(&event.target_id)
        .bind(metadata)
        .bind(&event.created_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn list_recent(&self, realm_id: &Uuid, limit: usize) -> Result<Vec<AuditEvent>> {
        let rows: Vec<AuditEventRow> = sqlx::query_as(
            "SELECT id, realm_id, actor_user_id, action, target_type, target_id, metadata, created_at
             FROM audit_events
             WHERE realm_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(realm_id.to_string())
        .bind(limit as i64)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| AuditEvent {
                id: Uuid::parse_str(&row.id).unwrap_or_else(|_| Uuid::nil()),
                realm_id: Uuid::parse_str(&row.realm_id).unwrap_or_else(|_| Uuid::nil()),
                actor_user_id: row
                    .actor_user_id
                    .and_then(|value| Uuid::parse_str(&value).ok()),
                action: row.action,
                target_type: row.target_type,
                target_id: row.target_id,
                metadata: serde_json::from_str(&row.metadata).unwrap_or(Value::Null),
                created_at: row.created_at,
            })
            .collect())
    }
}
