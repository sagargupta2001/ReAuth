use crate::domain::log::{LogEntry, LogPublisher};
use chrono::Utc;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::Subscriber;
use tracing_subscriber::{layer::Context, Layer};

const SPAN_EVENT_MESSAGE: &str = "trace.span";

#[derive(Clone)]
struct SpanData {
    enabled: bool,
    trace_id: Option<String>,
    span_id: Option<String>,
    parent_id: Option<String>,
    request_id: Option<String>,
    user_id: Option<String>,
    realm: Option<String>,
    method: Option<String>,
    route: Option<String>,
    path: Option<String>,
    status: Option<i64>,
    name: String,
    start: Instant,
    start_time: chrono::DateTime<chrono::Utc>,
}

/// Captures tracing spans as telemetry entries (for waterfall views).
pub struct TracingSpanAdapter {
    publisher: Arc<dyn LogPublisher>,
}

impl TracingSpanAdapter {
    pub fn new(publisher: Arc<dyn LogPublisher>) -> Self {
        Self { publisher }
    }
}

impl<S> Layer<S> for TracingSpanAdapter
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = HashMap::new();
        let mut visitor = FieldVisitor(&mut fields);
        attrs.record(&mut visitor);

        let span = match ctx.span(id) {
            Some(span) => span,
            None => return,
        };

        let parent_data = span
            .parent()
            .and_then(|parent| parent.extensions().get::<SpanData>().cloned());

        let telemetry_flag = fields
            .get("telemetry")
            .map(|value| value == "span" || value == "true")
            .unwrap_or(false);

        let trace_id = fields
            .get("trace_id")
            .cloned()
            .or_else(|| parent_data.as_ref().and_then(|data| data.trace_id.clone()));

        let span_id = fields
            .get("span_id")
            .cloned()
            .unwrap_or_else(generate_span_id);

        let parent_id = fields
            .get("parent_id")
            .cloned()
            .or_else(|| parent_data.as_ref().and_then(|data| data.span_id.clone()));

        let enabled = telemetry_flag && trace_id.is_some();

        let data = SpanData {
            enabled,
            trace_id,
            span_id: Some(span_id),
            parent_id,
            request_id: fields.get("request_id").cloned().or_else(|| {
                parent_data
                    .as_ref()
                    .and_then(|data| data.request_id.clone())
            }),
            user_id: fields
                .get("user_id")
                .cloned()
                .or_else(|| parent_data.as_ref().and_then(|data| data.user_id.clone())),
            realm: fields
                .get("realm")
                .cloned()
                .or_else(|| parent_data.as_ref().and_then(|data| data.realm.clone())),
            method: fields
                .get("method")
                .cloned()
                .or_else(|| parent_data.as_ref().and_then(|data| data.method.clone())),
            route: fields
                .get("route")
                .cloned()
                .or_else(|| parent_data.as_ref().and_then(|data| data.route.clone())),
            path: fields
                .get("path")
                .cloned()
                .or_else(|| parent_data.as_ref().and_then(|data| data.path.clone())),
            status: fields
                .get("status")
                .and_then(|value| value.parse::<i64>().ok())
                .or_else(|| parent_data.as_ref().and_then(|data| data.status)),
            name: span.metadata().name().to_string(),
            start: Instant::now(),
            start_time: Utc::now(),
        };

        span.extensions_mut().insert(data);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let span = match ctx.span(id) {
            Some(span) => span,
            None => return,
        };

        let mut fields = HashMap::new();
        let mut visitor = FieldVisitor(&mut fields);
        values.record(&mut visitor);

        let mut extensions = span.extensions_mut();
        if let Some(data) = extensions.get_mut::<SpanData>() {
            apply_field_update(data, &fields);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = match ctx.span(&id) {
            Some(span) => span,
            None => return,
        };

        let data = match span.extensions().get::<SpanData>() {
            Some(data) => data.clone(),
            None => return,
        };

        if !data.enabled {
            return;
        }

        let trace_id = match data.trace_id {
            Some(trace_id) => trace_id,
            None => return,
        };

        let span_id = match data.span_id {
            Some(span_id) => span_id,
            None => return,
        };

        let duration_ms = data.start.elapsed().as_millis() as i64;
        let start_time = data.start_time.to_rfc3339();
        let timestamp = Utc::now().to_rfc3339();

        let mut fields = HashMap::new();
        fields.insert("trace_id".to_string(), trace_id);
        fields.insert("span_id".to_string(), span_id);
        if let Some(parent_id) = data.parent_id {
            fields.insert("parent_id".to_string(), parent_id);
        }
        fields.insert("name".to_string(), data.name);
        fields.insert("duration_ms".to_string(), duration_ms.to_string());
        fields.insert("start_time".to_string(), start_time);

        if let Some(request_id) = data.request_id {
            fields.insert("request_id".to_string(), request_id);
        }
        if let Some(user_id) = data.user_id {
            if !user_id.is_empty() {
                fields.insert("user_id".to_string(), user_id);
            }
        }
        if let Some(realm) = data.realm {
            if !realm.is_empty() {
                fields.insert("realm".to_string(), realm);
            }
        }
        if let Some(method) = data.method {
            fields.insert("method".to_string(), method);
        }
        if let Some(route) = data.route {
            fields.insert("route".to_string(), route);
        }
        if let Some(path) = data.path {
            fields.insert("path".to_string(), path);
        }
        if let Some(status) = data.status {
            fields.insert("status".to_string(), status.to_string());
        }

        let entry = LogEntry {
            timestamp,
            level: "TRACE".to_string(),
            target: "trace.span".to_string(),
            message: SPAN_EVENT_MESSAGE.to_string(),
            fields,
        };

        let publisher = self.publisher.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                publisher.publish(entry).await;
            });
        }
    }
}

fn apply_field_update(data: &mut SpanData, fields: &HashMap<String, String>) {
    if let Some(value) = fields.get("request_id") {
        data.request_id = Some(value.clone());
    }
    if let Some(value) = fields.get("user_id") {
        data.user_id = Some(value.clone());
    }
    if let Some(value) = fields.get("realm") {
        data.realm = Some(value.clone());
    }
    if let Some(value) = fields.get("method") {
        data.method = Some(value.clone());
    }
    if let Some(value) = fields.get("route") {
        data.route = Some(value.clone());
    }
    if let Some(value) = fields.get("path") {
        data.path = Some(value.clone());
    }
    if let Some(value) = fields
        .get("status")
        .and_then(|value| value.parse::<i64>().ok())
    {
        data.status = Some(value);
    }
    if let Some(value) = fields.get("trace_id") {
        data.trace_id = Some(value.clone());
    }
    if let Some(value) = fields.get("span_id") {
        data.span_id = Some(value.clone());
    }
    if let Some(value) = fields.get("parent_id") {
        data.parent_id = Some(value.clone());
    }
}

fn generate_span_id() -> String {
    let mut bytes = [0u8; 8];
    let mut rng = rand::rngs::OsRng;
    loop {
        rng.fill_bytes(&mut bytes);
        if bytes.iter().any(|b| *b != 0) {
            break;
        }
    }
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_char(byte >> 4));
        out.push(hex_char(byte & 0x0f));
    }
    out
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}

// A visitor to extract key-value fields from a tracing span.
struct FieldVisitor<'a>(&'a mut HashMap<String, String>);

impl Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    // NOTE: record_value is not available on the stable Visit trait used here.
}
