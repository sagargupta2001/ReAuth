use crate::domain::pagination::PageResponse;
use crate::domain::telemetry::{
    TelemetryLog, TelemetryLogQuery, TelemetryTrace, TelemetryTraceQuery,
};
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

    pub async fn list_logs(&self, query: TelemetryLogQuery) -> Result<PageResponse<TelemetryLog>> {
        self.repo.list_logs(query).await
    }

    pub async fn list_traces(
        &self,
        query: TelemetryTraceQuery,
    ) -> Result<PageResponse<TelemetryTrace>> {
        self.repo.list_traces(query).await
    }

    pub async fn list_trace_spans(&self, trace_id: &str) -> Result<Vec<TelemetryTrace>> {
        self.repo.list_trace_spans(trace_id).await
    }
}
