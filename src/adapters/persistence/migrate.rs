use anyhow::Result;
use sqlx::{Pool, Sqlite};

/// Run all pending migrations (from project root `migrations`)
pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?;

    tracing::info!("All migrations applied");
    Ok(())
}
