use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;

use crate::config::DatabaseConfig;

pub type Database = Arc<SqlitePool>;

pub async fn init_db(config: &DatabaseConfig) -> Result<Database> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .after_connect(|conn, _| {
            Box::pin(async move {
                // Safer defaults for SQLite under concurrent access.
                sqlx::query("PRAGMA foreign_keys = ON;")
                    .execute(&mut *conn)
                    .await?;
                let _ = sqlx::query("PRAGMA journal_mode = WAL;")
                    .execute(&mut *conn)
                    .await;
                sqlx::query("PRAGMA busy_timeout = 10000;")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&config.url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(Arc::new(pool))
}
