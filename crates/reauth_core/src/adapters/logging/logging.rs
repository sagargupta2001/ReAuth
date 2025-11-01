use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};
use once_cell::sync::Lazy;
use tracing::Level;

pub static LOGGER: Lazy<()> = Lazy::new(|| {
    // Read RUST_LOG env variable or default to info
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // fmt subscriber with colors and structured output
    let subscriber = fmt::fmt()
        .with_max_level(Level::TRACE) // capture all levels
        .with_target(false) // omit module path if desired
        .with_level(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .finish()
        .with(env_filter);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");
});
