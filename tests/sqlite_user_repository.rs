mod support;

use anyhow::Result;
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_user_repository::SqliteUserRepository;
use reauth::domain::pagination::{PageRequest, SortDirection};
use reauth::domain::user::User;
use reauth::ports::user_repository::UserRepository;
use support::TestDb;
use uuid::Uuid;

fn page_request(
    page: i64,
    per_page: i64,
    sort_dir: Option<SortDirection>,
    q: Option<&str>,
) -> PageRequest {
    PageRequest {
        page,
        per_page,
        sort_by: Some("username".to_string()),
        sort_dir,
        q: q.map(|value| value.to_string()),
    }
}

fn user(id: Uuid, realm_id: Uuid, username: &str, hashed_password: &str) -> User {
    User {
        id,
        realm_id,
        username: username.to_string(),
        hashed_password: hashed_password.to_string(),
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

#[tokio::test]
async fn save_and_find_users_by_id_and_username() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteUserRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-users").await?;
    insert_realm(&db.pool, other_realm, "realm-other").await?;

    let alice = user(Uuid::new_v4(), realm_id, "alice", "hash1");
    repo.save(&alice, None).await?;

    let by_id = repo.find_by_id(&alice.id).await?;
    assert_eq!(by_id.unwrap().username, "alice");

    let by_username = repo.find_by_username(&realm_id, "alice").await?;
    assert_eq!(by_username.unwrap().id, alice.id);

    let other = repo.find_by_username(&other_realm, "alice").await?;
    assert!(other.is_none());

    let missing = repo.find_by_id(&Uuid::new_v4()).await?;
    assert!(missing.is_none());
    Ok(())
}

#[tokio::test]
async fn update_user_persists_changes() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteUserRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-update").await?;

    let mut alice = user(Uuid::new_v4(), realm_id, "alice", "hash1");
    repo.save(&alice, None).await?;

    alice.username = "alice-updated".to_string();
    alice.hashed_password = "hash2".to_string();
    repo.update(&alice, None).await?;

    let updated = repo.find_by_id(&alice.id).await?.unwrap();
    assert_eq!(updated.username, "alice-updated");
    assert_eq!(updated.hashed_password, "hash2");
    Ok(())
}

#[tokio::test]
async fn list_users_with_filters_sorting_and_pagination() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteUserRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();

    insert_realm(&db.pool, realm_id, "realm-list").await?;
    insert_realm(&db.pool, other_realm, "realm-other").await?;

    let alice = user(Uuid::new_v4(), realm_id, "alice", "hash");
    let bob = user(Uuid::new_v4(), realm_id, "bob", "hash");
    let carol = user(Uuid::new_v4(), realm_id, "carol", "hash");
    let zara = user(Uuid::new_v4(), other_realm, "zara", "hash");

    for u in [&alice, &bob, &carol, &zara] {
        repo.save(u, None).await?;
    }

    let page1 = repo
        .list(
            &realm_id,
            &page_request(1, 2, Some(SortDirection::Asc), None),
        )
        .await?;
    assert_eq!(page1.meta.total, 3);
    assert_eq!(page1.data.len(), 2);
    assert_eq!(page1.data[0].username, "alice");
    assert_eq!(page1.data[1].username, "bob");

    let page2 = repo
        .list(
            &realm_id,
            &page_request(2, 2, Some(SortDirection::Asc), None),
        )
        .await?;
    assert_eq!(page2.meta.total, 3);
    assert_eq!(page2.data.len(), 1);
    assert_eq!(page2.data[0].username, "carol");

    let desc_page = repo
        .list(
            &realm_id,
            &page_request(1, 3, Some(SortDirection::Desc), None),
        )
        .await?;
    assert_eq!(desc_page.data[0].username, "carol");
    assert_eq!(desc_page.data[1].username, "bob");

    let filtered = repo
        .list(
            &realm_id,
            &page_request(1, 10, Some(SortDirection::Asc), Some("bo")),
        )
        .await?;
    assert_eq!(filtered.meta.total, 1);
    assert_eq!(filtered.data[0].username, "bob");

    let empty_query = repo
        .list(
            &realm_id,
            &page_request(1, 10, Some(SortDirection::Asc), Some("")),
        )
        .await?;
    assert_eq!(empty_query.meta.total, 3);
    Ok(())
}
