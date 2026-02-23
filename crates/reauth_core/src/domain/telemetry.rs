use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::pagination::PageRequest;

#[derive(Debug, Serialize, Clone)]
pub struct TelemetryLog {
    pub id: Uuid,
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Value,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_id: Option<String>,
    pub user_id: Option<String>,
    pub realm: Option<String>,
    pub method: Option<String>,
    pub route: Option<String>,
    pub path: Option<String>,
    pub status: Option<i64>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TelemetryTrace {
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: String,
    pub duration_ms: i64,
    pub status: Option<i64>,
    pub method: Option<String>,
    pub route: Option<String>,
    pub path: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub realm: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelemetryLogQuery {
    pub page: PageRequest,
    pub level: Option<String>,
    pub target: Option<String>,
    pub search: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub include_spans: bool,
}

#[derive(Debug, Clone)]
pub struct TelemetryTraceQuery {
    pub page: PageRequest,
    pub search: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}
