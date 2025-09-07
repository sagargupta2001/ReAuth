use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::path::Path;
use anyhow::Result;

pub type Database = Arc<SqlitePool>;

pub async fn init_db() -> Result<Database> {
    // Absolute path is safer
    let data_dir = "./data";
    let db_file = format!("{}/reauth.db", data_dir);

    // Ensure the folder exists
    let path = Path::new(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| anyhow::anyhow!("Failed to create data directory: {}", e))?;
    }

    // Check if we can create/open the database file
    let db_path = Path::new(&db_file);
    if !db_path.exists() {
        std::fs::File::create(&db_path)
            .map_err(|e| anyhow::anyhow!("Failed to create database file: {}", e))?;
    }

    // SQLite connection string
    let db_url = format!("sqlite://{}", db_file);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(Arc::new(pool))
}
