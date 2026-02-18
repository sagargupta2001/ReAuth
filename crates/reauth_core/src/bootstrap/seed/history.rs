use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::ports::transaction_manager::Transaction;
use sqlx::{Row, SqlitePool};

#[derive(Debug)]
pub struct SeedRecord {
    pub version: i32,
    pub checksum: String,
}

#[derive(Debug)]
pub struct SeedStatus {
    pub name: String,
    pub version: i32,
    pub checksum: String,
    pub applied_at: String,
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

    pub async fn upsert(
        &self,
        name: &str,
        version: i32,
        checksum: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        if let Some(tx) = tx.and_then(SqliteTransaction::from_trait) {
            let executor = tx.as_mut();
            sqlx::query(
                "INSERT INTO seed_history (name, version, checksum) VALUES (?, ?, ?)\n                 ON CONFLICT(name) DO UPDATE SET version = excluded.version, checksum = excluded.checksum, applied_at = CURRENT_TIMESTAMP",
            )
            .bind(name)
            .bind(version)
            .bind(checksum)
            .execute(executor)
            .await?;
            return Ok(());
        }

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

    pub async fn list_all(&self) -> anyhow::Result<Vec<SeedStatus>> {
        let rows = sqlx::query(
            "SELECT name, version, checksum, applied_at FROM seed_history ORDER BY name",
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SeedStatus {
                name: row.get::<String, _>("name"),
                version: row.get::<i64, _>("version") as i32,
                checksum: row.get::<String, _>("checksum"),
                applied_at: row.get::<String, _>("applied_at"),
            })
            .collect())
    }
}
