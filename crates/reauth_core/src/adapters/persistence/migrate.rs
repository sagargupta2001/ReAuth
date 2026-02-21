use anyhow::Result;
use sqlx::{Pool, Sqlite};

/// Run all pending migrations (from workspace root `../migrations`)
pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    // go one directory up from `reauth_core/`
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?;

    tracing::info!("All migrations applied");
    Ok(())
}
