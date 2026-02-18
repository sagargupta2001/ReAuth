use crate::adapters::eventing::log_broadcast_bus::LogBroadcastBus;
use crate::adapters::logging::tracing_adapter::TracingLogAdapter;
use crate::config::Settings;
use std::sync::Arc;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_logging(settings: &Settings) -> Arc<LogBroadcastBus> {
    let log_bus = Arc::new(LogBroadcastBus::new());
    let adapter = TracingLogAdapter::new(log_bus.clone());

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if !settings.logging.filter.trim().is_empty() {
            EnvFilter::new(settings.logging.filter.clone())
        } else if !settings.logging.level.trim().is_empty() {
            EnvFilter::new(settings.logging.level.clone())
        } else {
            EnvFilter::new("info")
        }
    });

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(adapter);

    // Avoid panicking if a global subscriber is already set (common in tests).
    let _ = subscriber.try_init();

    log_bus
}
