mod database;
mod server;
mod banner;
mod logging;
mod status;

use once_cell::sync::Lazy;
use tracing::info;
use crate::banner::print_banner;
use crate::database::init_db;
use crate::server::start_server;
use crate::logging::LOGGER;
use crate::status::log_system_status;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&LOGGER);
    // Load .env file and initialize logger
    dotenvy::dotenv().ok();

    print_banner();

    info!("Initializing database...");
    let db = init_db().await?;

    log_system_status("http://localhost:3000", Some("http://localhost:5173"), "Up & Running");

    info!("Starting server...");
    start_server(db).await?;

    Ok(())
}