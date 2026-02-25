use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::domain::pagination::{PageResponse, SortDirection};
use crate::domain::telemetry::{
    DeliveryLog, DeliveryLogQuery, TelemetryLog, TelemetryLogQuery, TelemetryTrace,
    TelemetryTraceQuery,
};
use crate::error::{Error, Result};
use crate::ports::telemetry_repository::TelemetryRepository;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{FromRow, QueryBuilder, Row, Sqlite};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteTelemetryRepository {
    pool: TelemetryDatabase,
}

impl SqliteTelemetryRepository {
    pub fn new(pool: TelemetryDatabase) -> Self {
        Self { pool }
    }

    fn apply_log_filters(builder: &mut QueryBuilder<Sqlite>, query: &TelemetryLogQuery) {
        let mut has_filter = false;

        let push_where = |builder: &mut QueryBuilder<Sqlite>, has_filter: &mut bool| {
            if *has_filter {
                builder.push(" AND ");
            } else {
                builder.push(" WHERE ");
                *has_filter = true;
            }
        };

        if let Some(level) = &query.level {
            push_where(builder, &mut has_filter);
            builder.push("level = ");
            builder.push_bind(level.clone());
        }

        if let Some(target) = &query.target {
            push_where(builder, &mut has_filter);
            builder.push("target = ");
            builder.push_bind(target.clone());
        }

        if let Some(start_time) = &query.start_time {
            push_where(builder, &mut has_filter);
            builder.push("timestamp >= ");
            builder.push_bind(start_time.clone());
        }

        if let Some(end_time) = &query.end_time {
            push_where(builder, &mut has_filter);
            builder.push("timestamp <= ");
            builder.push_bind(end_time.clone());
        }

        if !query.include_spans {
            push_where(builder, &mut has_filter);
            builder.push("(target IS NULL OR target != ");
            builder.push_bind("trace.span".to_string());
            builder.push(")");
        }

        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search);
            push_where(builder, &mut has_filter);
            builder.push("(");
            builder.push("message LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR fields LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR target LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR trace_id LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR request_id LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR route LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR path LIKE ");
            builder.push_bind(pattern);
            builder.push(")");
        }
    }

    fn apply_trace_filters(builder: &mut QueryBuilder<Sqlite>, query: &TelemetryTraceQuery) {
        let mut has_filter = false;
        let push_where = |builder: &mut QueryBuilder<Sqlite>, has_filter: &mut bool| {
            if *has_filter {
                builder.push(" AND ");
            } else {
                builder.push(" WHERE ");
                *has_filter = true;
            }
        };

        push_where(builder, &mut has_filter);
        builder.push("method IS NOT NULL");

        push_where(builder, &mut has_filter);
        builder.push("(parent_id IS NULL OR parent_id = '')");

        if let Some(start_time) = &query.start_time {
            push_where(builder, &mut has_filter);
            builder.push("start_time >= ");
            builder.push_bind(start_time.clone());
        }

        if let Some(end_time) = &query.end_time {
            push_where(builder, &mut has_filter);
            builder.push("start_time <= ");
            builder.push_bind(end_time.clone());
        }

        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search);
            push_where(builder, &mut has_filter);
            builder.push("(");
            builder.push("name LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR route LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR path LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR trace_id LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR request_id LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR method LIKE ");
            builder.push_bind(pattern);
            builder.push(")");
        }
    }

    fn apply_delivery_filters(builder: &mut QueryBuilder<Sqlite>, query: &DeliveryLogQuery) {
        let mut has_filter = false;
        let push_where = |builder: &mut QueryBuilder<Sqlite>, has_filter: &mut bool| {
            if *has_filter {
                builder.push(" AND ");
            } else {
                builder.push(" WHERE ");
                *has_filter = true;
            }
        };

        if let Some(realm_id) = &query.realm_id {
            push_where(builder, &mut has_filter);
            builder.push("realm_id = ");
            builder.push_bind(realm_id.to_string());
        }

        if let Some(target_type) = &query.target_type {
            push_where(builder, &mut has_filter);
            builder.push("target_type = ");
            builder.push_bind(target_type.clone());
        }

        if let Some(target_id) = &query.target_id {
            push_where(builder, &mut has_filter);
            builder.push("target_id = ");
            builder.push_bind(target_id.clone());
        }

        if let Some(event_type) = &query.event_type {
            push_where(builder, &mut has_filter);
            builder.push("event_type = ");
            builder.push_bind(event_type.clone());
        }

        if let Some(event_id) = &query.event_id {
            push_where(builder, &mut has_filter);
            builder.push("event_id = ");
            builder.push_bind(event_id.clone());
        }

        if let Some(failed) = query.failed {
            push_where(builder, &mut has_filter);
            if failed {
                builder.push("error IS NOT NULL");
            } else {
                builder.push("error IS NULL");
            }
        }

        if let Some(start_time) = &query.start_time {
            push_where(builder, &mut has_filter);
            builder.push("delivered_at >= ");
            builder.push_bind(start_time.clone());
        }

        if let Some(end_time) = &query.end_time {
            push_where(builder, &mut has_filter);
            builder.push("delivered_at <= ");
            builder.push_bind(end_time.clone());
        }
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

#[derive(Debug, FromRow)]
struct DeliveryLogRow {
    id: String,
    event_id: String,
    realm_id: Option<String>,
    target_type: String,
    target_id: String,
    event_type: String,
    event_version: String,
    attempt: i64,
    payload: String,
    payload_compressed: i64,
    response_status: Option<i64>,
    response_body: Option<String>,
    error: Option<String>,
    latency_ms: Option<i64>,
    delivered_at: String,
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
    async fn list_logs(&self, query: TelemetryLogQuery) -> Result<PageResponse<TelemetryLog>> {
        let page = query.page.page.max(1);
        let limit = query.page.per_page.clamp(1, 200);
        let offset = (page - 1) * limit;

        let mut count_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM telemetry_logs");
        Self::apply_log_filters(&mut count_builder, &query);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, timestamp, level, target, message, fields, request_id, trace_id, span_id, parent_id, user_id, realm, method, route, path, status, duration_ms FROM telemetry_logs",
        );
        Self::apply_log_filters(&mut builder, &query);

        let sort_col = match query.page.sort_by.as_deref() {
            Some("timestamp") => "timestamp",
            Some("duration_ms") => "duration_ms",
            Some("status") => "status",
            Some("level") => "level",
            _ => "timestamp",
        };
        let sort_dir = match query.page.sort_dir.unwrap_or(SortDirection::Desc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        builder.push(" LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let rows: Vec<TelemetryLogRow> = builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let data = rows
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
            .collect();

        Ok(PageResponse::new(data, total, page, limit))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_traces", db_op = "select")
    )]
    async fn list_traces(
        &self,
        query: TelemetryTraceQuery,
    ) -> Result<PageResponse<TelemetryTrace>> {
        let page = query.page.page.max(1);
        let limit = query.page.per_page.clamp(1, 200);
        let offset = (page - 1) * limit;

        let mut count_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM telemetry_traces");
        Self::apply_trace_filters(&mut count_builder, &query);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT trace_id, span_id, parent_id, name, start_time, duration_ms, status, method, route, path, request_id, user_id, realm
             FROM telemetry_traces",
        );
        Self::apply_trace_filters(&mut builder, &query);

        let sort_col = match query.page.sort_by.as_deref() {
            Some("start_time") => "start_time",
            Some("duration_ms") => "duration_ms",
            Some("status") => "status",
            _ => "start_time",
        };
        let sort_dir = match query.page.sort_dir.unwrap_or(SortDirection::Desc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));
        builder.push(" LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        let rows: Vec<TelemetryTraceRow> = builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let data = rows
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
            .collect();

        Ok(PageResponse::new(data, total, page, limit))
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "delivery_logs", db_op = "select")
    )]
    async fn list_delivery_logs(
        &self,
        query: DeliveryLogQuery,
    ) -> Result<PageResponse<DeliveryLog>> {
        let page = query.page.page.max(1);
        let limit = query.page.per_page.clamp(1, 200);
        let offset = (page - 1) * limit;

        let mut count_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM delivery_logs");
        Self::apply_delivery_filters(&mut count_builder, &query);
        let total: i64 = count_builder
            .build()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?
            .get(0);

        let mut data_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, event_id, realm_id, target_type, target_id, event_type, event_version,
                    attempt, payload, payload_compressed, response_status, response_body, error,
                    latency_ms, delivered_at
             FROM delivery_logs",
        );
        Self::apply_delivery_filters(&mut data_builder, &query);
        data_builder.push(" ORDER BY delivered_at DESC LIMIT ");
        data_builder.push_bind(limit);
        data_builder.push(" OFFSET ");
        data_builder.push_bind(offset);

        let rows: Vec<DeliveryLogRow> = data_builder
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let data = rows
            .into_iter()
            .map(|row| DeliveryLog {
                id: row.id,
                event_id: row.event_id,
                realm_id: row.realm_id.and_then(|value| Uuid::parse_str(&value).ok()),
                target_type: row.target_type,
                target_id: row.target_id,
                event_type: row.event_type,
                event_version: row.event_version,
                attempt: row.attempt,
                payload: row.payload,
                payload_compressed: row.payload_compressed == 1,
                response_status: row.response_status,
                response_body: row.response_body,
                error: row.error,
                latency_ms: row.latency_ms,
                delivered_at: row.delivered_at,
            })
            .collect();

        Ok(PageResponse::new(data, total, page, limit))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "delivery_logs", db_op = "select")
    )]
    async fn get_delivery_log(&self, delivery_id: &str) -> Result<Option<DeliveryLog>> {
        let row: Option<DeliveryLogRow> = sqlx::query_as(
            "SELECT id, event_id, realm_id, target_type, target_id, event_type, event_version,
                    attempt, payload, payload_compressed, response_status, response_body, error,
                    latency_ms, delivered_at
             FROM delivery_logs
             WHERE id = ?",
        )
        .bind(delivery_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| DeliveryLog {
            id: row.id,
            event_id: row.event_id,
            realm_id: row.realm_id.and_then(|value| Uuid::parse_str(&value).ok()),
            target_type: row.target_type,
            target_id: row.target_id,
            event_type: row.event_type,
            event_version: row.event_version,
            attempt: row.attempt,
            payload: row.payload,
            payload_compressed: row.payload_compressed == 1,
            response_status: row.response_status,
            response_body: row.response_body,
            error: row.error,
            latency_ms: row.latency_ms,
            delivered_at: row.delivered_at,
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_logs", db_op = "delete")
    )]
    async fn delete_logs_before(&self, before: Option<&str>) -> Result<i64> {
        let result = if let Some(before) = before {
            sqlx::query("DELETE FROM telemetry_logs WHERE timestamp < ?")
                .bind(before)
                .execute(self.pool.as_ref())
                .await
        } else {
            sqlx::query("DELETE FROM telemetry_logs")
                .execute(self.pool.as_ref())
                .await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected() as i64)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "telemetry_traces", db_op = "delete")
    )]
    async fn delete_traces_before(&self, before: Option<&str>) -> Result<i64> {
        let result = if let Some(before) = before {
            sqlx::query("DELETE FROM telemetry_traces WHERE start_time < ?")
                .bind(before)
                .execute(self.pool.as_ref())
                .await
        } else {
            sqlx::query("DELETE FROM telemetry_traces")
                .execute(self.pool.as_ref())
                .await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected() as i64)
    }
}
