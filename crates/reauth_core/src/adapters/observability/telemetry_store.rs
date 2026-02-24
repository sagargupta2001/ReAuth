use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;

pub type TelemetryDatabase = Arc<SqlitePool>;

pub async fn init_telemetry_db(path: &str) -> Result<TelemetryDatabase> {
    let clean_path = sanitize_path(path);
    if clean_path != ":memory:" {
        ensure_parent_dir(&clean_path)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(&clean_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA journal_mode = WAL;")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA synchronous = NORMAL;")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA temp_store = MEMORY;")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA busy_timeout = 10000;")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await?;

    create_schema(&pool).await?;

    Ok(Arc::new(pool))
}

fn sanitize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return ":memory:".to_string();
    }

    trimmed
        .strip_prefix("sqlite:")
        .unwrap_or(trimmed)
        .to_string()
}

fn ensure_parent_dir(path: &str) -> Result<()> {
    let parent = Path::new(path).parent();
    if let Some(parent) = parent {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

async fn create_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS telemetry_logs (
            id TEXT PRIMARY KEY NOT NULL,
            timestamp TEXT NOT NULL,
            level TEXT NOT NULL,
            target TEXT NOT NULL,
            message TEXT NOT NULL,
            fields TEXT NOT NULL DEFAULT '{}',
            request_id TEXT,
            trace_id TEXT,
            span_id TEXT,
            parent_id TEXT,
            user_id TEXT,
            realm TEXT,
            method TEXT,
            route TEXT,
            path TEXT,
            status INTEGER,
            duration_ms INTEGER
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS telemetry_traces (
            trace_id TEXT NOT NULL,
            span_id TEXT NOT NULL,
            parent_id TEXT,
            name TEXT NOT NULL,
            start_time TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            status INTEGER,
            method TEXT,
            route TEXT,
            path TEXT,
            request_id TEXT,
            user_id TEXT,
            realm TEXT,
            PRIMARY KEY (trace_id, span_id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_logs_timestamp ON telemetry_logs (timestamp)",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_telemetry_logs_level ON telemetry_logs (level)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_telemetry_logs_trace ON telemetry_logs (trace_id)")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_logs_request ON telemetry_logs (request_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_traces_start_time ON telemetry_traces (start_time)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_traces_trace ON telemetry_traces (trace_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
