use crate::domain::log::{LogEntry, LogSubscriber};
use crate::domain::telemetry::{TelemetryLog, TelemetryTrace};
use crate::ports::telemetry_repository::TelemetryRepository;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, warn};
use uuid::Uuid;

pub struct TelemetryWriter {
    repo: Arc<dyn TelemetryRepository>,
}

impl TelemetryWriter {
    pub fn new(repo: Arc<dyn TelemetryRepository>) -> Self {
        Self { repo }
    }

    pub fn spawn(self, subscriber: Arc<dyn LogSubscriber>) {
        let mut receiver = subscriber.subscribe();
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(entry) => {
                        if let Err(err) = self.persist(entry).await {
                            error!("Telemetry writer failed to persist log: {:?}", err);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!("Telemetry writer lagging behind log stream.");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    async fn persist(&self, entry: LogEntry) -> crate::error::Result<()> {
        let is_span_event = entry.message == "trace.span" || entry.target == "trace.span";
        let request_id = field_value(&entry.fields, "request_id");
        let trace_id = field_value(&entry.fields, "trace_id");
        let span_id = field_value(&entry.fields, "span_id");
        let parent_id = field_value(&entry.fields, "parent_id");
        let user_id = field_value(&entry.fields, "user_id");
        let realm = field_value(&entry.fields, "realm");
        let method = field_value(&entry.fields, "method");
        let route = field_value(&entry.fields, "route");
        let path = field_value(&entry.fields, "path");
        let status = field_value(&entry.fields, "status").and_then(|v| v.parse::<i64>().ok());
        let duration_ms =
            field_value(&entry.fields, "duration_ms").and_then(|v| v.parse::<i64>().ok());

        if !is_span_event {
            let log = TelemetryLog {
                id: Uuid::new_v4(),
                timestamp: entry.timestamp.clone(),
                level: entry.level,
                target: entry.target,
                message: entry.message.clone(),
                fields: serde_json::to_value(&entry.fields).unwrap_or_default(),
                request_id: request_id.clone(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                parent_id: parent_id.clone(),
                user_id: user_id.clone(),
                realm: realm.clone(),
                method: method.clone(),
                route: route.clone(),
                path: path.clone(),
                status,
                duration_ms,
            };

            self.repo.insert_log(&log).await?;
        }

        if entry.message == "api.request" {
            if let (Some(trace_id), Some(span_id), Some(duration_ms)) =
                (trace_id.clone(), span_id.clone(), duration_ms)
            {
                let name = route
                    .clone()
                    .or_else(|| path.clone())
                    .unwrap_or(entry.message);
                let start_time = calculate_start_time(&entry.timestamp, duration_ms);

                let trace = TelemetryTrace {
                    trace_id,
                    span_id,
                    parent_id: parent_id.clone(),
                    name,
                    start_time,
                    duration_ms,
                    status,
                    method: method.clone(),
                    route: route.clone(),
                    path: path.clone(),
                    request_id: request_id.clone(),
                    user_id: user_id.clone(),
                    realm: realm.clone(),
                };

                self.repo.insert_trace(&trace).await?;
            }
        }

        if is_span_event {
            if let (Some(trace_id), Some(span_id), Some(duration_ms)) =
                (trace_id, span_id, duration_ms)
            {
                let name = field_value(&entry.fields, "name")
                    .or_else(|| route.clone())
                    .or_else(|| path.clone())
                    .unwrap_or_else(|| "span".to_string());
                let start_time = field_value(&entry.fields, "start_time")
                    .unwrap_or_else(|| calculate_start_time(&entry.timestamp, duration_ms));

                let trace = TelemetryTrace {
                    trace_id,
                    span_id,
                    parent_id,
                    name,
                    start_time,
                    duration_ms,
                    status,
                    method,
                    route,
                    path,
                    request_id,
                    user_id,
                    realm,
                };

                self.repo.insert_trace(&trace).await?;
            }
        }

        Ok(())
    }
}

fn field_value(fields: &HashMap<String, String>, key: &str) -> Option<String> {
    fields
        .get(key)
        .map(|value| value.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn calculate_start_time(timestamp: &str, duration_ms: i64) -> String {
    let end_time = DateTime::parse_from_rfc3339(timestamp)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    (end_time - Duration::milliseconds(duration_ms)).to_rfc3339()
}
