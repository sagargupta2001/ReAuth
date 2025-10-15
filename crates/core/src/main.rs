mod database;
mod server;
mod logging;
mod config;

use std::env;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use tracing::info;
use reauth_plugin_manager::{ManagerConfig, PluginManager};
use crate::config::Settings;
use crate::database::init_db;
use crate::database::migrate::run_migrations;
use crate::logging::banner::print_banner;
use crate::logging::logging::LOGGER;
use crate::logging::status::log_system_status;
use crate::server::start_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&LOGGER);
    dotenvy::dotenv().ok();
    print_banner();

    let settings = Settings::new()?;

    let manager_config = ManagerConfig {
        handshake_timeout_secs: settings.plugins.handshake_timeout_secs,
    };

    let exe_path = env::current_exe()?;
    let is_dev_run = exe_path.ancestors().any(|p| p.ends_with("target"));
    let plugins_path = if is_dev_run {
        PathBuf::from("plugins")
    } else {
        let mut prod_path = exe_path;
        prod_path.pop();
        prod_path.join("plugins")
    };
    let plugins_path_clone = plugins_path.clone();
    info!("Loading plugins from: {:?}", plugins_path.canonicalize().unwrap_or_else(|_| plugins_path.clone()));

    // Initialize and run the plugin manager
    let plugin_manager = PluginManager::new(manager_config);
    let manager_clone = plugin_manager.clone();
    tokio::spawn(async move {
        if let Some(path_str) = plugins_path.to_str() {
            manager_clone.discover_and_run(path_str).await;
        }
    });

    info!("Initializing database...");
    let db = init_db().await?;

    let server_url = format!(
        "{}://{}:{}",
        settings.server.scheme,
        settings.server.host,
        settings.server.port
    );
    let ui_location = if cfg!(feature = "embed-ui") { None } else { Some(settings.ui.dev_url.as_str()) };
    log_system_status(&server_url, "Up & Running");

    if let Err(e) = run_migrations(&db).await {
        tracing::warn!("⚠Migration warning: {}", e);
    }

    info!("Starting server...");
    start_server(db, plugin_manager, settings, plugins_path_clone).await?;

    Ok(())
}