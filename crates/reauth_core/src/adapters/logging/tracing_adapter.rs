use crate::{domain::log_entry::LogEntry, ports::log_bus::LogPublisher};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::field::Field;
use tracing::{field::Visit, Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

/// A custom tracing layer that captures structured logs and
/// publishes them to the LogPublisher port.
pub struct TracingLogAdapter {
    publisher: Arc<dyn LogPublisher>,
}

impl TracingLogAdapter {
    pub fn new(publisher: Arc<dyn LogPublisher>) -> Self {
        Self { publisher }
    }
}

impl<S> Layer<S> for TracingLogAdapter
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = HashMap::new();
        let mut visitor = FieldVisitor(&mut fields);
        event.record(&mut visitor);

        let log_entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: fields.remove("message").unwrap_or_default(),
            fields,
        };

        // Publish to the event bus.
        // We must spawn a task because on_event is synchronous.
        let publisher = self.publisher.clone();
        tokio::spawn(async move {
            publisher.publish(log_entry).await;
        });
    }
}

// A simple visitor to extract key-value fields from a tracing event.
struct FieldVisitor<'a>(&'a mut HashMap<String, String>);
impl Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
    // ... you can add more `record_` types (u64, i64, bool) here
}
