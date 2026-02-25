use crate::domain::pagination::PageResponse;
use crate::domain::telemetry::{
    DeliveryLog, DeliveryLogQuery, TelemetryLog, TelemetryLogQuery, TelemetryTrace,
    TelemetryTraceQuery,
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

    pub async fn list_delivery_logs(
        &self,
        query: DeliveryLogQuery,
    ) -> Result<PageResponse<DeliveryLog>> {
        self.repo.list_delivery_logs(query).await
    }

    pub async fn get_delivery_log(&self, delivery_id: &str) -> Result<Option<DeliveryLog>> {
        self.repo.get_delivery_log(delivery_id).await
    }

    pub async fn clear_logs(&self, before: Option<&str>) -> Result<i64> {
        self.repo.delete_logs_before(before).await
    }

    pub async fn clear_traces(&self, before: Option<&str>) -> Result<i64> {
        self.repo.delete_traces_before(before).await
    }
}
