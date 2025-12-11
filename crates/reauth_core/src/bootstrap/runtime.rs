use crate::adapters::start_server;
use crate::initialize;
use tracing::info;

/// Starts the full ReAuth Core application (normal mode).
pub async fn run() -> anyhow::Result<()> {
    let app_state = initialize().await?;

    let server_url = format!(
        "{}://{}:{}",
        app_state.settings.server.scheme,
        app_state.settings.server.host,
        app_state.settings.server.port
    );

    info!("Server started at: {}", server_url);
    info!("Database status: {}", "Up & Running");

    start_server(app_state).await?;

    Ok(())
}
