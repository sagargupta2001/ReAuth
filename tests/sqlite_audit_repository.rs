mod support;

use anyhow::Result;
use chrono::Utc;
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_audit_repository::SqliteAuditRepository;
use reauth::domain::audit::AuditEvent;
use reauth::ports::audit_repository::AuditRepository;
use serde_json::json;
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
async fn insert_and_list_recent_audit_events() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteAuditRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-audit").await?;

    let event1 = AuditEvent {
        id: Uuid::new_v4(),
        realm_id,
        actor_user_id: None,
        action: "user.login".to_string(),
        target_type: "user".to_string(),
        target_id: Some("user-123".to_string()),
        metadata: json!({"ip": "127.0.0.1"}),
        created_at: Utc::now().to_rfc3339(),
    };

    repo.insert(&event1).await?;

    let events = repo.list_recent(&realm_id, 10).await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event1.id);
    assert_eq!(events[0].action, "user.login");
    assert_eq!(events[0].target_type, "user");
    assert_eq!(events[0].metadata["ip"], "127.0.0.1");

    Ok(())
}
