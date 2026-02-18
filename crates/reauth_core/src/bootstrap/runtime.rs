use crate::adapters::start_server;
use crate::initialize;
use tracing::info;

/// Starts the full ReAuth Core application (normal mode).
pub async fn run() -> anyhow::Result<()> {
    let app_state = initialize().await?;

    info!("Database status: {}", "Up & Running");

    start_server(app_state).await?;

    Ok(())
}
