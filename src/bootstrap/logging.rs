use crate::adapters::eventing::log_broadcast_bus::LogBroadcastBus;
use crate::adapters::logging::span_adapter::TracingSpanAdapter;
use crate::adapters::logging::tracing_adapter::TracingLogAdapter;
use crate::config::Settings;
use std::sync::Arc;
use time::format_description::parse;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_logging(settings: &Settings) -> Arc<LogBroadcastBus> {
    let log_bus = Arc::new(LogBroadcastBus::new());
    let adapter = TracingLogAdapter::new(log_bus.clone());
    let span_adapter = TracingSpanAdapter::new(log_bus.clone());

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if !settings.logging.filter.trim().is_empty() {
            EnvFilter::new(settings.logging.filter.clone())
        } else if !settings.logging.level.trim().is_empty() {
            EnvFilter::new(settings.logging.level.clone())
        } else {
            EnvFilter::new("info")
        }
    });

    let time_format = parse("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]Z")
        .expect("invalid time format");

    if settings.logging.json {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(settings.logging.show_target)
            .with_current_span(settings.logging.show_span_context)
            .with_span_list(settings.logging.show_span_list)
            .with_timer(UtcTime::new(time_format));

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(adapter)
            .with(span_adapter);

        let _ = subscriber.try_init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_target(settings.logging.show_target)
            .with_timer(UtcTime::new(time_format));

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(adapter)
            .with(span_adapter);

        let _ = subscriber.try_init();
    }

    log_bus
}
