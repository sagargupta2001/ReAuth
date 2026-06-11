mod support;

use anyhow::Result;
use chrono::Utc;
use reauth::adapters::persistence::sqlite_user_email_repository::SqliteUserEmailRepository;
use reauth::adapters::persistence::sqlite_user_repository::SqliteUserRepository;
use reauth::domain::user::{User, EMPTY_METADATA_JSON};
use reauth::domain::user_email::UserEmail;
use reauth::ports::user_email_repository::UserEmailRepository;
use reauth::ports::user_repository::UserRepository;
use support::TestDb;
use uuid::Uuid;

fn make_user(realm_id: Uuid, username: &str) -> User {
    User {
        id: Uuid::new_v4(),
        realm_id,
        username: username.to_string(),
        first_name: None,
        last_name: None,
        hashed_password: "hash".to_string(),
        public_metadata_json: EMPTY_METADATA_JSON.to_string(),
        private_metadata_json: EMPTY_METADATA_JSON.to_string(),
        unsafe_metadata_json: EMPTY_METADATA_JSON.to_string(),
        force_password_reset: false,
        password_login_disabled: false,
        created_at: Some(Utc::now()),
        updated_at: None,
        last_sign_in_at: None,
        locked_until: None,
        banned_at: None,
    }
}

async fn insert_realm(db: &TestDb, realm_id: Uuid, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)",
    )
    .bind(realm_id.to_string())
    .bind(name)
    .bind(900_i64)
    .bind(604800_i64)
    .execute(&*db.pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn save_and_find_by_user_id() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm1").await?;
    let user = make_user(realm_id, "alice");
    user_repo.save(&user, None).await?;

    let e1 = UserEmail::new(
        user.id,
        realm_id,
        "alice@example.com".to_string(),
        true,
        false,
    );
    let e2 = UserEmail::new(
        user.id,
        realm_id,
        "alice@work.com".to_string(),
        false,
        false,
    );
    email_repo.save(&e1, None).await?;
    email_repo.save(&e2, None).await?;

    let emails = email_repo.find_by_user_id(&user.id).await?;
    assert_eq!(emails.len(), 2);
    // Primary comes first due to ORDER BY is_primary DESC
    assert!(emails[0].is_primary);
    assert_eq!(emails[0].email, "alice@example.com");
    Ok(())
}

#[tokio::test]
async fn find_by_email_case_insensitive() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm2").await?;
    let user = make_user(realm_id, "bob");
    user_repo.save(&user, None).await?;

    let email = UserEmail::new(
        user.id,
        realm_id,
        "Bob@Example.COM".to_string(),
        true,
        false,
    );
    email_repo.save(&email, None).await?;

    // find_by_email uses email_normalized, so pass lowercase
    let found = email_repo
        .find_by_email(&realm_id, "bob@example.com")
        .await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().user_id, user.id);

    // Different realm returns None
    let other_realm = Uuid::new_v4();
    let not_found = email_repo
        .find_by_email(&other_realm, "bob@example.com")
        .await?;
    assert!(not_found.is_none());
    Ok(())
}

#[tokio::test]
async fn find_primary_returns_primary_email() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm3").await?;
    let user = make_user(realm_id, "carol");
    user_repo.save(&user, None).await?;

    let secondary = UserEmail::new(
        user.id,
        realm_id,
        "carol-secondary@example.com".to_string(),
        false,
        false,
    );
    let primary = UserEmail::new(
        user.id,
        realm_id,
        "carol@example.com".to_string(),
        true,
        false,
    );
    email_repo.save(&secondary, None).await?;
    email_repo.save(&primary, None).await?;

    let found = email_repo.find_primary(&user.id).await?;
    assert!(found.is_some());
    let p = found.unwrap();
    assert_eq!(p.email, "carol@example.com");
    assert!(p.is_primary);
    Ok(())
}

#[tokio::test]
async fn find_primary_returns_none_when_no_emails() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm4").await?;
    let user = make_user(realm_id, "nobody");
    user_repo.save(&user, None).await?;

    let result = email_repo.find_primary(&user.id).await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test]
async fn set_primary_demotes_old_primary_via_trigger() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm5").await?;
    let user = make_user(realm_id, "dave");
    user_repo.save(&user, None).await?;

    let e1 = UserEmail::new(user.id, realm_id, "dave@first.com".to_string(), true, true);
    let e2 = UserEmail::new(
        user.id,
        realm_id,
        "dave@second.com".to_string(),
        false,
        true,
    );
    email_repo.save(&e1, None).await?;
    email_repo.save(&e2, None).await?;

    // Promote e2 to primary — trigger should demote e1
    email_repo.set_primary(&user.id, &e2.id, None).await?;

    let emails = email_repo.find_by_user_id(&user.id).await?;
    let primaries: Vec<_> = emails.iter().filter(|e| e.is_primary).collect();
    assert_eq!(primaries.len(), 1, "exactly one primary after set_primary");
    assert_eq!(primaries[0].id, e2.id);

    let old_primary = emails.iter().find(|e| e.id == e1.id).unwrap();
    assert!(!old_primary.is_primary, "old primary was demoted");
    Ok(())
}

#[tokio::test]
async fn set_verified_updates_flag() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm6").await?;
    let user = make_user(realm_id, "eve");
    user_repo.save(&user, None).await?;

    let email = UserEmail::new(
        user.id,
        realm_id,
        "eve@example.com".to_string(),
        true,
        false,
    );
    email_repo.save(&email, None).await?;

    // Verify: false -> true
    email_repo.set_verified(&email.id, true, None).await?;
    let found = email_repo.find_primary(&user.id).await?.unwrap();
    assert!(found.is_verified);

    // Unverify: true -> false
    email_repo.set_verified(&email.id, false, None).await?;
    let found2 = email_repo.find_primary(&user.id).await?.unwrap();
    assert!(!found2.is_verified);
    Ok(())
}

#[tokio::test]
async fn delete_removes_email() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm7").await?;
    let user = make_user(realm_id, "frank");
    user_repo.save(&user, None).await?;

    let e1 = UserEmail::new(
        user.id,
        realm_id,
        "frank@example.com".to_string(),
        true,
        false,
    );
    let e2 = UserEmail::new(user.id, realm_id, "frank@alt.com".to_string(), false, false);
    email_repo.save(&e1, None).await?;
    email_repo.save(&e2, None).await?;

    email_repo.delete(&e2.id, None).await?;

    let emails = email_repo.find_by_user_id(&user.id).await?;
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].id, e1.id);
    Ok(())
}

#[tokio::test]
async fn uniqueness_constraint_per_realm() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm8a").await?;
    insert_realm(&db, other_realm, "realm8b").await?;

    let user1 = make_user(realm_id, "grace");
    let user2 = make_user(realm_id, "heidi");
    let user3 = make_user(other_realm, "ivan");
    user_repo.save(&user1, None).await?;
    user_repo.save(&user2, None).await?;
    user_repo.save(&user3, None).await?;

    // First save succeeds
    let e1 = UserEmail::new(
        user1.id,
        realm_id,
        "shared@example.com".to_string(),
        true,
        false,
    );
    email_repo.save(&e1, None).await?;

    // Same email in same realm for different user -> unique constraint violation
    let e2 = UserEmail::new(
        user2.id,
        realm_id,
        "shared@example.com".to_string(),
        true,
        false,
    );
    let result = email_repo.save(&e2, None).await;
    assert!(result.is_err(), "duplicate email in same realm should fail");

    // Same email in different realm -> allowed
    let e3 = UserEmail::new(
        user3.id,
        other_realm,
        "shared@example.com".to_string(),
        true,
        false,
    );
    email_repo.save(&e3, None).await?;
    Ok(())
}

#[tokio::test]
async fn trigger_enforces_single_primary_on_insert() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm9").await?;
    let user = make_user(realm_id, "judy");
    user_repo.save(&user, None).await?;

    let e1 = UserEmail::new(user.id, realm_id, "judy@first.com".to_string(), true, false);
    email_repo.save(&e1, None).await?;

    // Insert a second primary — trigger should demote e1
    let e2 = UserEmail::new(
        user.id,
        realm_id,
        "judy@second.com".to_string(),
        true,
        false,
    );
    email_repo.save(&e2, None).await?;

    let emails = email_repo.find_by_user_id(&user.id).await?;
    let primaries: Vec<_> = emails.iter().filter(|e| e.is_primary).collect();
    assert_eq!(primaries.len(), 1);
    assert_eq!(primaries[0].id, e2.id);
    Ok(())
}

#[tokio::test]
async fn isolation_between_users() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let email_repo = SqliteUserEmailRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm10").await?;

    let user_a = make_user(realm_id, "userA");
    let user_b = make_user(realm_id, "userB");
    user_repo.save(&user_a, None).await?;
    user_repo.save(&user_b, None).await?;

    let ea = UserEmail::new(user_a.id, realm_id, "a@example.com".to_string(), true, true);
    let eb = UserEmail::new(
        user_b.id,
        realm_id,
        "b@example.com".to_string(),
        true,
        false,
    );
    email_repo.save(&ea, None).await?;
    email_repo.save(&eb, None).await?;

    // Promoting user_a's primary should NOT affect user_b
    let ea2 = UserEmail::new(
        user_a.id,
        realm_id,
        "a2@example.com".to_string(),
        false,
        false,
    );
    email_repo.save(&ea2, None).await?;
    email_repo.set_primary(&user_a.id, &ea2.id, None).await?;

    let b_primary = email_repo.find_primary(&user_b.id).await?.unwrap();
    assert_eq!(b_primary.id, eb.id, "user_b primary unchanged");

    let a_emails = email_repo.find_by_user_id(&user_a.id).await?;
    let a_primaries: Vec<_> = a_emails.iter().filter(|e| e.is_primary).collect();
    assert_eq!(a_primaries.len(), 1);
    assert_eq!(a_primaries[0].id, ea2.id);
    Ok(())
}
