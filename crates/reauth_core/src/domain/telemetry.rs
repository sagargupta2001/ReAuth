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

#[derive(Debug, Serialize, Clone)]
pub struct DeliveryLog {
    pub id: String,
    pub event_id: String,
    pub realm_id: Option<Uuid>,
    pub target_type: String,
    pub target_id: String,
    pub event_type: String,
    pub event_version: String,
    pub attempt: i64,
    pub payload: String,
    pub payload_compressed: bool,
    pub response_status: Option<i64>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub error_chain: Option<String>,
    pub latency_ms: Option<i64>,
    pub delivered_at: String,
}

#[derive(Debug, Clone)]
pub struct DeliveryLogQuery {
    pub page: PageRequest,
    pub realm_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub event_type: Option<String>,
    pub event_id: Option<String>,
    pub failed: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeliveryMetricsAggregate {
    pub total_routed: i64,
    pub success_count: i64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EventRoutingMetrics {
    pub window_hours: i64,
    pub total_routed: i64,
    pub success_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub active_plugins: i64,
}
