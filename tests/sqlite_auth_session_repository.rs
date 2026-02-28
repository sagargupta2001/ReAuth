mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_auth_session_repository::SqliteAuthSessionRepository;
use reauth::domain::auth_session::{AuthenticationSession, SessionStatus};
use reauth::ports::auth_session_repository::AuthSessionRepository;
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
async fn create_update_find_and_delete_session() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteAuthSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-auth").await?;

    let flow_version_id = Uuid::new_v4();
    let mut session = AuthenticationSession::new(realm_id, flow_version_id, "start".to_string());
    session.context = json!({"step": "start"});

    repo.create(&session).await?;

    let fetched = repo.find_by_id(&session.id).await?.unwrap();
    assert_eq!(fetched.current_node_id, "start");
    assert_eq!(fetched.status, SessionStatus::Active);
    assert_eq!(fetched.context["step"], "start");

    let user_id = Uuid::new_v4();
    session.current_node_id = "next".to_string();
    session.status = SessionStatus::Completed;
    session.user_id = Some(user_id);
    session.context = json!({"step": "next"});

    repo.update(&session).await?;

    let updated = repo.find_by_id(&session.id).await?.unwrap();
    assert_eq!(updated.current_node_id, "next");
    assert_eq!(updated.status, SessionStatus::Completed);
    assert_eq!(updated.user_id, Some(user_id));
    assert_eq!(updated.context["step"], "next");

    repo.delete(&session.id).await?;
    let missing = repo.find_by_id(&session.id).await?;
    assert!(missing.is_none());
    Ok(())
}

#[tokio::test]
async fn find_by_id_maps_status_and_user_id() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteAuthSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-status").await?;

    let flow_version_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(10);
    let user_id = Uuid::new_v4();

    let rows = vec![
        ("active", SessionStatus::Active, None),
        ("completed", SessionStatus::Completed, Some(user_id)),
        ("failed", SessionStatus::Failed, None),
        ("unknown", SessionStatus::Failed, None),
    ];

    for (idx, (status, expected, maybe_user)) in rows.into_iter().enumerate() {
        let session_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO auth_sessions (id, realm_id, flow_version_id, current_node_id, context, status, user_id, created_at, updated_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(session_id.to_string())
        .bind(realm_id.to_string())
        .bind(flow_version_id.to_string())
        .bind(format!("node-{}", idx))
        .bind(json!({"idx": idx}).to_string())
        .bind(status)
        .bind(maybe_user.map(|id| id.to_string()))
        .bind(now)
        .bind(now)
        .bind(expires_at)
        .execute(&*db.pool)
        .await?;

        let fetched = repo.find_by_id(&session_id).await?.unwrap();
        assert_eq!(fetched.status, expected);
        if status == "completed" {
            assert_eq!(fetched.user_id, Some(user_id));
        } else {
            assert!(fetched.user_id.is_none());
        }
    }
    Ok(())
}

#[tokio::test]
async fn find_by_id_handles_invalid_uuid_fields() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteAuthSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-invalid").await?;

    let session_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(10);

    sqlx::query(
        "INSERT INTO auth_sessions (id, realm_id, flow_version_id, current_node_id, context, status, user_id, created_at, updated_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(session_id.to_string())
    .bind(realm_id.to_string())
    .bind("also-not-a-uuid")
    .bind("node")
    .bind(json!({"edge": true}).to_string())
    .bind("active")
    .bind("bad-user-id")
    .bind(now)
    .bind(now)
    .bind(expires_at)
    .execute(&*db.pool)
    .await?;

    let fetched = repo.find_by_id(&session_id).await?.unwrap();
    assert_eq!(fetched.realm_id, realm_id);
    assert_eq!(fetched.flow_version_id, Uuid::nil());
    assert_eq!(fetched.user_id, Some(Uuid::nil()));
    Ok(())
}

#[tokio::test]
async fn find_by_id_returns_none_when_missing() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteAuthSessionRepository::new(db.pool.clone());

    let missing = repo.find_by_id(&Uuid::new_v4()).await?;
    assert!(missing.is_none());
    Ok(())
}
