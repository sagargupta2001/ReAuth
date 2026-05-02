use crate::adapters::persistence::connection::Database;
use crate::domain::audit::{AuditActionCount, AuditEvent};
use crate::error::{Error, Result};
use crate::ports::audit_repository::AuditRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tracing::instrument;
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

#[derive(Debug, FromRow)]
struct AuditActionCountRow {
    action: String,
    count: i64,
}

impl AuditEventRow {
    fn into_domain(self) -> AuditEvent {
        AuditEvent {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::nil()),
            realm_id: Uuid::parse_str(&self.realm_id).unwrap_or_else(|_| Uuid::nil()),
            actor_user_id: self
                .actor_user_id
                .and_then(|value| Uuid::parse_str(&value).ok()),
            action: self.action,
            target_type: self.target_type,
            target_id: self.target_id,
            metadata: serde_json::from_str(&self.metadata).unwrap_or(Value::Null),
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl AuditRepository for SqliteAuditRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "audit_events", db_op = "insert")
    )]
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "audit_events", db_op = "select")
    )]
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

        Ok(rows.into_iter().map(AuditEventRow::into_domain).collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "audit_events", db_op = "count")
    )]
    async fn count_by_actions_since(
        &self,
        realm_id: &Uuid,
        actions: &[&str],
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditActionCount>> {
        if actions.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT action, COUNT(*) AS count
             FROM audit_events
             WHERE realm_id = ",
        );
        builder.push_bind(realm_id.to_string());
        builder.push(" AND action IN (");
        {
            let mut separated = builder.separated(", ");
            for action in actions {
                separated.push_bind((*action).to_string());
            }
        }
        builder.push(")");
        if let Some(since) = since {
            builder.push(" AND created_at >= ");
            builder.push_bind(since.to_rfc3339());
        }
        builder.push(" GROUP BY action");

        let rows: Vec<AuditActionCountRow> = builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| AuditActionCount {
                action: row.action,
                count: row.count.max(0) as u64,
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "audit_events", db_op = "select")
    )]
    async fn list_recent_by_actions(
        &self,
        realm_id: &Uuid,
        actions: &[&str],
        limit: usize,
    ) -> Result<Vec<AuditEvent>> {
        if actions.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, realm_id, actor_user_id, action, target_type, target_id, metadata, created_at
             FROM audit_events
             WHERE realm_id = ",
        );
        builder.push_bind(realm_id.to_string());
        builder.push(" AND action IN (");
        {
            let mut separated = builder.separated(", ");
            for action in actions {
                separated.push_bind((*action).to_string());
            }
        }
        builder.push(") ORDER BY created_at DESC LIMIT ");
        builder.push_bind(limit as i64);

        let rows: Vec<AuditEventRow> = builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows.into_iter().map(AuditEventRow::into_domain).collect())
    }
}
