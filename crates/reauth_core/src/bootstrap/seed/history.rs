use sqlx::{Row, SqlitePool};

#[derive(Debug)]
pub struct SeedRecord {
    pub version: i32,
    pub checksum: String,
}

pub struct SeedHistory<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SeedHistory<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, name: &str) -> anyhow::Result<Option<SeedRecord>> {
        let record = sqlx::query("SELECT version, checksum FROM seed_history WHERE name = ?")
            .bind(name)
            .fetch_optional(self.pool)
            .await?;

        Ok(record.map(|row| SeedRecord {
            version: row.get::<i64, _>("version") as i32,
            checksum: row.get::<String, _>("checksum"),
        }))
    }

    pub async fn upsert(&self, name: &str, version: i32, checksum: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO seed_history (name, version, checksum) VALUES (?, ?, ?)\n             ON CONFLICT(name) DO UPDATE SET version = excluded.version, checksum = excluded.checksum, applied_at = CURRENT_TIMESTAMP",
        )
        .bind(name)
        .bind(version)
        .bind(checksum)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
