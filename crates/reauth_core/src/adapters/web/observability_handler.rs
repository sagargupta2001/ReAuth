use crate::domain::pagination::{PageRequest, SortDirection};
use crate::domain::telemetry::{TelemetryLogQuery, TelemetryTraceQuery};
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub level: Option<String>,
    pub target: Option<String>,
    pub search: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub include_spans: Option<bool>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct TraceQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub search: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct CacheFlushPayload {
    pub namespace: Option<String>,
}

#[derive(Deserialize)]
pub struct CacheStatsQuery {
    pub namespace: Option<String>,
}

#[derive(Deserialize)]
pub struct TelemetryClearPayload {
    pub before: Option<String>,
}

const DEFAULT_LOG_LIMIT: i64 = 200;
const MAX_LOG_LIMIT: i64 = 1000;
const DEFAULT_TRACE_LIMIT: i64 = 200;
const MAX_TRACE_LIMIT: i64 = 1000;

// GET /api/system/observability/logs
pub async fn list_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<impl IntoResponse> {
    let mut page = query.page;
    if let Some(limit) = query.limit {
        page.per_page = limit.clamp(1, MAX_LOG_LIMIT);
        page.page = 1;
    }
    if page.per_page <= 0 {
        page.per_page = DEFAULT_LOG_LIMIT;
    }
    page.per_page = page.per_page.min(MAX_LOG_LIMIT);
    if page.sort_dir.is_none() {
        page.sort_dir = Some(SortDirection::Desc);
    }

    let search = query.search.or_else(|| page.q.clone());
    let (start, end) = normalize_time_range(query.start, query.end)?;
    let filter = TelemetryLogQuery {
        page,
        level: query.level.map(|value| value.to_uppercase()),
        target: query.target,
        search,
        start_time: start,
        end_time: end,
        include_spans: query.include_spans.unwrap_or(true),
    };

    let logs = state.telemetry_service.list_logs(filter).await?;
    Ok((StatusCode::OK, Json(logs)))
}

// GET /api/system/observability/traces
pub async fn list_traces_handler(
    State(state): State<AppState>,
    Query(query): Query<TraceQuery>,
) -> Result<impl IntoResponse> {
    let mut page = query.page;
    if let Some(limit) = query.limit {
        page.per_page = limit.clamp(1, MAX_TRACE_LIMIT);
        page.page = 1;
    }
    if page.per_page <= 0 {
        page.per_page = DEFAULT_TRACE_LIMIT;
    }
    page.per_page = page.per_page.min(MAX_TRACE_LIMIT);
    if page.sort_dir.is_none() {
        page.sort_dir = Some(SortDirection::Desc);
    }

    let search = query.search.or_else(|| page.q.clone());
    let (start, end) = normalize_time_range(query.start, query.end)?;
    let filter = TelemetryTraceQuery {
        page,
        search,
        start_time: start,
        end_time: end,
    };

    let traces = state.telemetry_service.list_traces(filter).await?;
    Ok((StatusCode::OK, Json(traces)))
}

// GET /api/system/observability/traces/{trace_id}
pub async fn list_trace_spans_handler(
    State(state): State<AppState>,
    axum::extract::Path(trace_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse> {
    let spans = state.telemetry_service.list_trace_spans(&trace_id).await?;
    Ok((StatusCode::OK, Json(spans)))
}

// GET /api/system/observability/cache/stats
pub async fn cache_stats_handler(
    State(state): State<AppState>,
    Query(query): Query<CacheStatsQuery>,
) -> Result<impl IntoResponse> {
    let stats = state.cache_service.stats_by_namespace().await;
    if let Some(namespace) = query.namespace {
        let stat = stats
            .into_iter()
            .find(|item| item.namespace == namespace)
            .ok_or_else(|| Error::NotFound(format!("Cache namespace not found: {}", namespace)))?;
        return Ok((StatusCode::OK, Json(serde_json::json!(stat))));
    }
    Ok((StatusCode::OK, Json(serde_json::json!(stats))))
}

// GET /api/system/observability/metrics
pub async fn metrics_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let snapshot = state.metrics_service.snapshot();
    Ok((StatusCode::OK, Json(snapshot)))
}

// POST /api/system/observability/logs/clear
pub async fn clear_logs_handler(
    State(state): State<AppState>,
    Json(payload): Json<TelemetryClearPayload>,
) -> Result<impl IntoResponse> {
    let before = payload
        .before
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .map(|value| value.to_rfc3339());

    let deleted = state
        .telemetry_service
        .clear_logs(before.as_deref())
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "deleted": deleted })),
    ))
}

// POST /api/system/observability/traces/clear
pub async fn clear_traces_handler(
    State(state): State<AppState>,
    Json(payload): Json<TelemetryClearPayload>,
) -> Result<impl IntoResponse> {
    let before = payload
        .before
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .map(|value| value.to_rfc3339());

    let deleted = state
        .telemetry_service
        .clear_traces(before.as_deref())
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "deleted": deleted })),
    ))
}

// POST /api/system/observability/cache/flush
pub async fn cache_flush_handler(
    State(state): State<AppState>,
    Json(payload): Json<CacheFlushPayload>,
) -> Result<impl IntoResponse> {
    let namespace = payload.namespace.as_deref().unwrap_or("all");
    if namespace == "all" {
        state.cache_service.clear_all().await;
    } else {
        let namespaces = state.cache_service.stats_by_namespace().await;
        let exists = namespaces.iter().any(|item| item.namespace == namespace);
        if !exists {
            return Err(Error::NotFound(format!(
                "Cache namespace not found: {}",
                namespace
            )));
        }
        state.cache_service.clear_namespace(namespace).await;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "flushed": namespace })),
    ))
}

fn normalize_time_range(
    start: Option<String>,
    end: Option<String>,
) -> Result<(Option<String>, Option<String>)> {
    let start_parsed = start
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .map(|value| value.to_rfc3339());
    let end_parsed = end
        .as_deref()
        .map(parse_rfc3339)
        .transpose()?
        .map(|value| value.to_rfc3339());

    if let (Some(start), Some(end)) = (&start_parsed, &end_parsed) {
        if start > end {
            return Err(Error::Validation(
                "start time must be before end time".to_string(),
            ));
        }
    }

    Ok((start_parsed, end_parsed))
}

fn parse_rfc3339(value: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| Error::Validation(format!("Invalid RFC3339 timestamp provided: {}", value)))
}
