use sqlx::{SqlitePool, Executor};
use std::fs;
use std::path::Path;
use anyhow::Result;

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migration_dir = Path::new("core/src/database/migrations");

    // Read all .sql files and sort by filename
    let mut entries: Vec<_> = fs::read_dir(migration_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let sql = fs::read_to_string(entry.path())?;
        pool.execute(sql.as_str()).await?;
        println!("Applied migration: {}", entry.path().display());
    }

    Ok(())
}
