mod database;
mod server;
mod logging;

use once_cell::sync::Lazy;
use tracing::info;
use reauth_plugin_manager::PluginManager;
use crate::database::init_db;
use crate::database::migrate::run_migrations;
use crate::logging::banner::print_banner;
use crate::logging::logging::LOGGER;
use crate::logging::status::log_system_status;
use crate::server::start_server;
use std::env;
use std::path::PathBuf;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&LOGGER);
    dotenvy::dotenv().ok();
    print_banner();

    // --- START OF REFACTORED LOGIC ---

    let exe_path = env::current_exe()?;

    // Check if the executable's path contains a 'target' directory.
    // This is a reliable way to distinguish a `cargo run` from a real deployment.
    let is_dev_run = exe_path.ancestors().any(|p| p.ends_with("target"));

    let plugins_path = if is_dev_run {
        // DEVELOPMENT mode (`cargo run`): The CWD is the workspace root.
        let dev_path = PathBuf::from("plugins");
        info!("Running in DEVELOPMENT mode. Loading plugins from: {:?}", dev_path.canonicalize().unwrap_or_default());
        dev_path
    } else {
        // PRODUCTION mode (standalone exe): Look for `plugins` next to the exe.
        let mut prod_path = exe_path;
        prod_path.pop(); // Remove the executable name to get its directory
        let prod_plugins_path = prod_path.join("plugins");
        info!("Running in PRODUCTION mode. Loading plugins from: {:?}", prod_plugins_path);
        prod_plugins_path
    };

    let plugin_manager = PluginManager::new();
    let manager_clone = plugin_manager.clone();
    tokio::spawn(async move {
        if let Some(path_str) = plugins_path.to_str() {
            manager_clone.discover_and_run(path_str).await;
        }
    });

    // --- END OF REFACTORED LOGIC ---

    info!("Initializing database...");
    let db = init_db().await?;

    log_system_status("http://localhost:3000", "Up & Running");

    if let Err(e) = run_migrations(&db).await {
        tracing::warn!("⚠️  Migration warning: {}", e);
    }

    info!("Starting server...");
    start_server(db, plugin_manager).await?;

    Ok(())
}