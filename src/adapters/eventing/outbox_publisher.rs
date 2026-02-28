use crate::adapters::persistence::connection::Database;
use crate::domain::events::{DomainEvent, EVENT_VERSION_V1};
use crate::ports::event_bus::EventPublisher;
use async_trait::async_trait;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct OutboxEventPublisher {
    db: Database,
}

impl OutboxEventPublisher {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventPublisher for OutboxEventPublisher {
    async fn publish(&self, event: DomainEvent) {
        let event_id = Uuid::new_v4();
        let occurred_at = Utc::now();
        let event_type = event.event_type().to_string();
        let payload_json = event.envelope_json(event_id, occurred_at, None, None);

        let result = sqlx::query(
            "INSERT INTO event_outbox (id, realm_id, event_type, event_version, occurred_at, payload_json, status, attempt_count, next_attempt_at)
             VALUES (?, ?, ?, ?, ?, ?, 'pending', 0, ?)",
        )
        .bind(event_id.to_string())
        .bind(None::<String>)
        .bind(event_type)
        .bind(EVENT_VERSION_V1)
        .bind(occurred_at.to_rfc3339())
        .bind(payload_json)
        .bind(occurred_at.to_rfc3339())
        .execute(&*self.db)
        .await;

        if let Err(err) = result {
            error!("Failed to write event to outbox: {}", err);
        }
    }
}
