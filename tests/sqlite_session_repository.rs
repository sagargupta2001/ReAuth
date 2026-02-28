mod support;

use anyhow::Result;
use chrono::{Duration, Utc};
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_session_repository::SqliteSessionRepository;
use reauth::domain::pagination::PageRequest;
use reauth::domain::session::RefreshToken;
use reauth::ports::session_repository::SessionRepository;
use support::TestDb;
use uuid::Uuid;

fn page_request(page: i64, per_page: i64, q: Option<&str>) -> PageRequest {
    PageRequest {
        page,
        per_page,
        sort_by: None,
        sort_dir: None,
        q: q.map(|value| value.to_string()),
    }
}

fn token(
    id: Uuid,
    user_id: Uuid,
    realm_id: Uuid,
    created_at: chrono::DateTime<Utc>,
) -> RefreshToken {
    RefreshToken {
        id,
        family_id: id,
        user_id,
        realm_id,
        client_id: None,
        expires_at: created_at + Duration::days(1),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("agent".to_string()),
        created_at,
        last_used_at: created_at,
        revoked_at: None,
        replaced_by: None,
    }
}

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

async fn insert_user(pool: &Database, user_id: Uuid, realm_id: Uuid, username: &str) -> Result<()> {
    sqlx::query("INSERT INTO users (id, realm_id, username, hashed_password) VALUES (?, ?, ?, ?)")
        .bind(user_id.to_string())
        .bind(realm_id.to_string())
        .bind(username)
        .bind("hash")
        .execute(&**pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn save_find_and_delete_refresh_token() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-session").await?;
    insert_user(&db.pool, user_id, realm_id, "alice").await?;

    let created_at = Utc::now();
    let refresh = token(Uuid::new_v4(), user_id, realm_id, created_at);
    repo.save(&refresh).await?;

    let fetched = repo.find_by_id(&refresh.id).await?.unwrap();
    assert_eq!(fetched.user_id, user_id);

    repo.delete_by_id(&refresh.id).await?;
    let missing = repo.find_by_id(&refresh.id).await?;
    assert!(missing.is_none());
    Ok(())
}

#[tokio::test]
async fn find_by_id_skips_expired_tokens() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-expired").await?;
    insert_user(&db.pool, user_id, realm_id, "bob").await?;

    let expired = RefreshToken {
        id: Uuid::new_v4(),
        family_id: Uuid::new_v4(),
        user_id,
        realm_id,
        client_id: None,
        expires_at: Utc::now() - Duration::minutes(5),
        ip_address: None,
        user_agent: None,
        created_at: Utc::now() - Duration::days(2),
        last_used_at: Utc::now() - Duration::days(2),
        revoked_at: None,
        replaced_by: None,
    };
    repo.save(&expired).await?;

    let fetched = repo.find_by_id(&expired.id).await?;
    assert!(fetched.is_none());
    Ok(())
}

#[tokio::test]
async fn list_refresh_tokens_with_filters_and_pagination() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteSessionRepository::new(db.pool.clone());

    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-list").await?;
    insert_realm(&db.pool, other_realm, "realm-other").await?;

    let user_id = Uuid::new_v4();
    insert_user(&db.pool, user_id, realm_id, "carol").await?;

    let now = Utc::now();
    let token_a = token(
        Uuid::new_v4(),
        user_id,
        realm_id,
        now - Duration::minutes(10),
    );
    let token_b = token(
        Uuid::new_v4(),
        user_id,
        realm_id,
        now - Duration::minutes(5),
    );
    let token_c = token(Uuid::new_v4(), user_id, realm_id, now);
    let other_token = token(Uuid::new_v4(), user_id, other_realm, now);
    let mut revoked_token = token(Uuid::new_v4(), user_id, realm_id, now);
    revoked_token.revoked_at = Some(Utc::now());
    let mut replaced_token = token(Uuid::new_v4(), user_id, realm_id, now);
    replaced_token.replaced_by = Some(Uuid::new_v4());

    for t in [
        &token_a,
        &token_b,
        &token_c,
        &other_token,
        &revoked_token,
        &replaced_token,
    ] {
        repo.save(t).await?;
    }

    let page1 = repo.list(&realm_id, &page_request(1, 2, None)).await?;
    assert_eq!(page1.meta.total, 3);
    assert_eq!(page1.data.len(), 2);
    assert_eq!(page1.data[0].id, token_c.id);
    assert_eq!(page1.data[1].id, token_b.id);

    let page2 = repo.list(&realm_id, &page_request(2, 2, None)).await?;
    assert_eq!(page2.data.len(), 1);
    assert_eq!(page2.data[0].id, token_a.id);

    let user_id_str = user_id.to_string();
    let filter = &user_id_str[user_id_str.len() - 6..];
    let filtered = repo
        .list(&realm_id, &page_request(1, 10, Some(filter)))
        .await?;
    assert_eq!(filtered.meta.total, 3);
    Ok(())
}
