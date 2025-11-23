use crate::adapters::eventing::log_broadcast_bus::LogBroadcastBus;
use crate::adapters::logging::tracing_adapter::TracingLogAdapter;
use std::sync::Arc;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_logging() -> Arc<LogBroadcastBus> {
    let log_bus = Arc::new(LogBroadcastBus::new());
    let adapter = TracingLogAdapter::new(log_bus.clone());

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(adapter)
        .init();

    log_bus
}
