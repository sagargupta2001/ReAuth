use crate::config::DatabaseConfig;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::path::Path;
use anyhow::Result;

pub type Database = Arc<SqlitePool>;

pub async fn init_db(config: &DatabaseConfig) -> Result<Database> {
    let data_dir = &config.data_dir;
    let db_file = Path::new(data_dir).join(&config.db_file);

    // Ensure the folder exists
    if let Some(parent) = db_file.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("Failed to create data directory: {}", e))?;
        }
    }

    let db_url = format!("sqlite://{}", db_file.to_str().unwrap());

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(Arc::new(pool))
}