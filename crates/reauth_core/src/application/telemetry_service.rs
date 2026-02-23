use crate::domain::telemetry::{TelemetryLog, TelemetryLogFilter, TelemetryTrace};
use crate::error::Result;
use crate::ports::telemetry_repository::TelemetryRepository;
use std::sync::Arc;

pub struct TelemetryService {
    repo: Arc<dyn TelemetryRepository>,
}

impl TelemetryService {
    pub fn new(repo: Arc<dyn TelemetryRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_logs(&self, filter: TelemetryLogFilter) -> Result<Vec<TelemetryLog>> {
        self.repo.list_logs(filter).await
    }

    pub async fn list_traces(&self, limit: usize) -> Result<Vec<TelemetryTrace>> {
        self.repo.list_traces(limit).await
    }

    pub async fn list_trace_spans(&self, trace_id: &str) -> Result<Vec<TelemetryTrace>> {
        self.repo.list_trace_spans(trace_id).await
    }
}
