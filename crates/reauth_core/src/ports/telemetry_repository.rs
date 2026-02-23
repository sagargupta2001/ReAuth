use crate::domain::telemetry::{TelemetryLog, TelemetryLogFilter, TelemetryTrace};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TelemetryRepository: Send + Sync {
    async fn insert_log(&self, log: &TelemetryLog) -> Result<()>;
    async fn insert_trace(&self, trace: &TelemetryTrace) -> Result<()>;
    async fn list_logs(&self, filter: TelemetryLogFilter) -> Result<Vec<TelemetryLog>>;
    async fn list_traces(&self, limit: usize) -> Result<Vec<TelemetryTrace>>;
    async fn list_trace_spans(&self, trace_id: &str) -> Result<Vec<TelemetryTrace>>;
}
