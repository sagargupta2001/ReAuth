use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::domain::telemetry::{TelemetryLog, TelemetryLogFilter, TelemetryTrace};
use crate::error::{Error, Result};
use crate::ports::telemetry_repository::TelemetryRepository;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteTelemetryRepository {
    pool: TelemetryDatabase,
}

impl SqliteTelemetryRepository {
    pub fn new(pool: TelemetryDatabase) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct TelemetryLogRow {
    id: String,
    timestamp: String,
    level: String,
    target: String,
    message: String,
    fields: String,
    request_id: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
    parent_id: Option<String>,
    user_id: Option<String>,
    realm: Option<String>,
    method: Option<String>,
    route: Option<String>,
    path: Option<String>,
    status: Option<i64>,
    duration_ms: Option<i64>,
}

#[derive(Debug, FromRow)]
struct TelemetryTraceRow {
    trace_id: String,
    span_id: String,
    parent_id: Option<String>,
    name: String,
    start_time: String,
    duration_ms: i64,
    status: Option<i64>,
    method: Option<String>,
    route: Option<String>,
    path: Option<String>,
    request_id: Option<String>,
    user_id: Option<String>,
    realm: Option<String>,
}

#[async_trait]
impl TelemetryRepository for SqliteTelemetryRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_logs", db_op = "insert")
    )]
    async fn insert_log(&self, log: &TelemetryLog) -> Result<()> {
        let fields = serde_json::to_string(&log.fields).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            "INSERT INTO telemetry_logs (
                id, timestamp, level, target, message, fields, request_id, trace_id, span_id,
                parent_id, user_id, realm, method, route, path, status, duration_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(log.id.to_string())
        .bind(&log.timestamp)
        .bind(&log.level)
        .bind(&log.target)
        .bind(&log.message)
        .bind(fields)
        .bind(&log.request_id)
        .bind(&log.trace_id)
        .bind(&log.span_id)
        .bind(&log.parent_id)
        .bind(&log.user_id)
        .bind(&log.realm)
        .bind(&log.method)
        .bind(&log.route)
        .bind(&log.path)
        .bind(log.status)
        .bind(log.duration_ms)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_traces", db_op = "insert")
    )]
    async fn insert_trace(&self, trace: &TelemetryTrace) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO telemetry_traces (
                trace_id, span_id, parent_id, name, start_time, duration_ms, status, method,
                route, path, request_id, user_id, realm
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&trace.trace_id)
        .bind(&trace.span_id)
        .bind(&trace.parent_id)
        .bind(&trace.name)
        .bind(&trace.start_time)
        .bind(trace.duration_ms)
        .bind(trace.status)
        .bind(&trace.method)
        .bind(&trace.route)
        .bind(&trace.path)
        .bind(&trace.request_id)
        .bind(&trace.user_id)
        .bind(&trace.realm)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_logs", db_op = "select")
    )]
    async fn list_logs(&self, filter: TelemetryLogFilter) -> Result<Vec<TelemetryLog>> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, timestamp, level, target, message, fields, request_id, trace_id, span_id, parent_id, user_id, realm, method, route, path, status, duration_ms FROM telemetry_logs",
        );

        let mut has_filter = false;
        if let Some(level) = filter.level {
            builder.push(" WHERE level = ");
            builder.push_bind(level);
            has_filter = true;
        }

        if let Some(search) = filter.search {
            let clause = " (message LIKE ";
            builder.push(if has_filter { " AND" } else { " WHERE" });
            builder.push(clause);
            builder.push_bind(format!("%{}%", search));
            builder.push(" OR fields LIKE ");
            builder.push_bind(format!("%{}%", search));
            builder.push(")");
        }

        builder.push(" ORDER BY timestamp DESC LIMIT ");
        builder.push_bind(filter.limit as i64);

        let rows: Vec<TelemetryLogRow> = builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| TelemetryLog {
                id: Uuid::parse_str(&row.id).unwrap_or_else(|_| Uuid::nil()),
                timestamp: row.timestamp,
                level: row.level,
                target: row.target,
                message: row.message,
                fields: serde_json::from_str(&row.fields).unwrap_or(Value::Null),
                request_id: row.request_id,
                trace_id: row.trace_id,
                span_id: row.span_id,
                parent_id: row.parent_id,
                user_id: row.user_id,
                realm: row.realm,
                method: row.method,
                route: row.route,
                path: row.path,
                status: row.status,
                duration_ms: row.duration_ms,
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_traces", db_op = "select")
    )]
    async fn list_traces(&self, limit: usize) -> Result<Vec<TelemetryTrace>> {
        let rows: Vec<TelemetryTraceRow> = sqlx::query_as(
            "SELECT trace_id, span_id, parent_id, name, start_time, duration_ms, status, method, route, path, request_id, user_id, realm
             FROM telemetry_traces
             ORDER BY start_time DESC
             LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| TelemetryTrace {
                trace_id: row.trace_id,
                span_id: row.span_id,
                parent_id: row.parent_id,
                name: row.name,
                start_time: row.start_time,
                duration_ms: row.duration_ms,
                status: row.status,
                method: row.method,
                route: row.route,
                path: row.path,
                request_id: row.request_id,
                user_id: row.user_id,
                realm: row.realm,
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_traces", db_op = "select")
    )]
    async fn list_trace_spans(&self, trace_id: &str) -> Result<Vec<TelemetryTrace>> {
        let rows: Vec<TelemetryTraceRow> = sqlx::query_as(
            "SELECT trace_id, span_id, parent_id, name, start_time, duration_ms, status, method, route, path, request_id, user_id, realm
             FROM telemetry_traces
             WHERE trace_id = ?
             ORDER BY start_time ",
        )
        .bind(trace_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| TelemetryTrace {
                trace_id: row.trace_id,
                span_id: row.span_id,
                parent_id: row.parent_id,
                name: row.name,
                start_time: row.start_time,
                duration_ms: row.duration_ms,
                status: row.status,
                method: row.method,
                route: row.route,
                path: row.path,
                request_id: row.request_id,
                user_id: row.user_id,
                realm: row.realm,
            })
            .collect())
    }
}
