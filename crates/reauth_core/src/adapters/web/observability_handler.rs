use crate::domain::telemetry::TelemetryLogFilter;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct TraceQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct CacheFlushPayload {
    pub namespace: Option<String>,
}

#[derive(Deserialize)]
pub struct CacheStatsQuery {
    pub namespace: Option<String>,
}

const DEFAULT_LOG_LIMIT: usize = 200;
const MAX_LOG_LIMIT: usize = 1000;
const DEFAULT_TRACE_LIMIT: usize = 200;
const MAX_TRACE_LIMIT: usize = 1000;

// GET /api/system/observability/logs
pub async fn list_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<impl IntoResponse> {
    let limit = query.limit.unwrap_or(DEFAULT_LOG_LIMIT).min(MAX_LOG_LIMIT);
    let filter = TelemetryLogFilter {
        level: query.level,
        search: query.search,
        limit,
    };

    let logs = state.telemetry_service.list_logs(filter).await?;
    Ok((StatusCode::OK, Json(logs)))
}

// GET /api/system/observability/traces
pub async fn list_traces_handler(
    State(state): State<AppState>,
    Query(query): Query<TraceQuery>,
) -> Result<impl IntoResponse> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_TRACE_LIMIT)
        .min(MAX_TRACE_LIMIT);
    let traces = state.telemetry_service.list_traces(limit).await?;
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
