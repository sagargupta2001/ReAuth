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

    start_server(
        app_state.settings,
        app_state.plugin_manager,
        app_state.plugins_path,
        app_state.user_service,
        app_state.rbac_service,
        app_state.auth_service,
        app_state.realm_service,
        app_state.log_subscriber,
        app_state.flow_engine,
    )
    .await?;

    Ok(())
}
