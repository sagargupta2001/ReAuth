mod support;

use anyhow::Result;
use chrono::Utc;
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_outbox_repository::SqliteOutboxRepository;
use reauth::domain::events::EventEnvelope;
use reauth::ports::outbox_repository::OutboxRepository;
use support::TestDb;
use uuid::Uuid;

async fn insert_realm(pool: &Database, realm_id: Uuid, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)",
    )
    .bind(realm_id.to_string())
    .bind(name)
    .bind(900_i64)
    .bind(604800_i64)
    .execute(&**pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn insert_outbox_event() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteOutboxRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-outbox").await?;

    let envelope = EventEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "test.event".to_string(),
        event_version: "v1".to_string(),
        occurred_at: Utc::now().to_rfc3339(),
        realm_id: Some(realm_id),
        actor: None,
        data: serde_json::json!({"key": "value"}),
    };

    repo.insert(&envelope, None).await?;

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM event_outbox WHERE id = ?")
        .bind(&envelope.event_id)
        .fetch_one(&*db.pool)
        .await?;

    assert_eq!(count.0, 1);

    Ok(())
}
