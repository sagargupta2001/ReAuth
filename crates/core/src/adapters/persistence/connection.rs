use crate::config::DatabaseConfig;
use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::Arc;

pub type Database = Arc<SqlitePool>;

pub async fn init_db(config: &DatabaseConfig) -> Result<Database> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(Arc::new(pool))
}