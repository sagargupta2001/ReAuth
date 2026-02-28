use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::events::EventEnvelope;
use crate::error::{Error, Result};
use crate::ports::outbox_repository::OutboxRepository;
use crate::ports::transaction_manager::Transaction;
use anyhow::anyhow;
use async_trait::async_trait;
use tracing::instrument;

pub struct SqliteOutboxRepository {
    pool: Database,
}

impl SqliteOutboxRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OutboxRepository for SqliteOutboxRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "event_outbox", db_op = "insert")
    )]
    async fn insert(
        &self,
        envelope: &EventEnvelope,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO event_outbox (id, realm_id, event_type, event_version, occurred_at, payload_json, status, attempt_count, next_attempt_at)
             VALUES (?, ?, ?, ?, ?, ?, 'pending', 0, ?)",
        )
        .bind(&envelope.event_id)
        .bind(envelope.realm_id.map(|id| id.to_string()))
        .bind(&envelope.event_type)
        .bind(&envelope.event_version)
        .bind(&envelope.occurred_at)
        .bind(serde_json::to_string(envelope).unwrap_or_else(|_| "{}".to_string()))
        .bind(&envelope.occurred_at);

        match tx {
            Some(tx) => {
                let sql_tx = SqliteTransaction::from_trait(tx)
                    .ok_or_else(|| Error::Unexpected(anyhow!("Invalid TX type")))?;
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
            None => {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }

        Ok(())
    }
}
