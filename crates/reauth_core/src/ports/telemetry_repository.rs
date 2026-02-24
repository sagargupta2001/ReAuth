use crate::domain::pagination::PageResponse;
use crate::domain::telemetry::{
    TelemetryLog, TelemetryLogQuery, TelemetryTrace, TelemetryTraceQuery,
};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TelemetryRepository: Send + Sync {
    async fn insert_log(&self, log: &TelemetryLog) -> Result<()>;
    async fn insert_trace(&self, trace: &TelemetryTrace) -> Result<()>;
    async fn list_logs(&self, query: TelemetryLogQuery) -> Result<PageResponse<TelemetryLog>>;
    async fn list_traces(&self, query: TelemetryTraceQuery)
        -> Result<PageResponse<TelemetryTrace>>;
    async fn list_trace_spans(&self, trace_id: &str) -> Result<Vec<TelemetryTrace>>;
    async fn delete_logs_before(&self, before: Option<&str>) -> Result<i64>;
    async fn delete_traces_before(&self, before: Option<&str>) -> Result<i64>;
}
