mod database;
mod server;
mod banner;

use tracing::info;
use crate::banner::print_banner;
use crate::database::init_db;
use crate::server::start_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file and initialize logger
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    print_banner();

    info!("Initializing database...");
    let db = init_db().await?;

    info!("Starting server...");
    start_server(db).await?;

    Ok(())
}