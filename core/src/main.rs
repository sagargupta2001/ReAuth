mod database;
mod server;
mod logging;

use once_cell::sync::Lazy;
use tracing::info;
use crate::database::init_db;
use crate::database::migrate::run_migrations;
use crate::logging::banner::print_banner;
use crate::logging::logging::LOGGER;
use crate::logging::status::log_system_status;
use crate::server::start_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&LOGGER);
    // Load .env file and initialize logger
    dotenvy::dotenv().ok();

    print_banner();

    info!("Initializing database...");
    let db = init_db().await?;

    log_system_status("http://localhost:3000", Some("http://localhost:5173"), "Up & Running");


    if let Err(e) = run_migrations(&db).await {
        tracing::warn!("⚠️  Migration warning: {}", e);
    }

    info!("Starting server...");
    start_server(db).await?;

    Ok(())
}