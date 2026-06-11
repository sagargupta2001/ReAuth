mod support;

use anyhow::Result;
use chrono::Utc;
use reauth::adapters::persistence::sqlite_user_phone_number_repository::SqliteUserPhoneNumberRepository;
use reauth::adapters::persistence::sqlite_user_repository::SqliteUserRepository;
use reauth::domain::user::User;
use reauth::domain::user_phone_number::UserPhoneNumber;
use reauth::ports::user_phone_number_repository::UserPhoneNumberRepository;
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
        force_password_reset: false,
        password_login_disabled: false,
        created_at: Some(Utc::now()),
        updated_at: None,
        last_sign_in_at: None,
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
async fn save_find_and_promote_primary_phone_numbers() -> Result<()> {
    let db = TestDb::new().await;
    let user_repo = SqliteUserRepository::new(db.pool.clone());
    let phone_repo = SqliteUserPhoneNumberRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();

    insert_realm(&db, realm_id, "realm-phone").await?;
    let user = make_user(realm_id, "alice");
    user_repo.save(&user, None).await?;

    let first = UserPhoneNumber::new(user.id, realm_id, "+1 (555) 0100".to_string(), true, false);
    let second = UserPhoneNumber::new(user.id, realm_id, "+1 555 0101".to_string(), false, false);

    phone_repo.save(&first, None).await?;
    phone_repo.save(&second, None).await?;

    let found = phone_repo
        .find_by_phone_number(&realm_id, "+15550100")
        .await?
        .expect("phone number should exist");
    assert_eq!(found.id, first.id);

    phone_repo.set_primary(&user.id, &second.id, None).await?;

    let all = phone_repo.find_by_user_id(&user.id).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, second.id);
    assert!(all[0].is_primary);
    assert!(!all[1].is_primary);

    phone_repo.set_verified(&second.id, true, None).await?;
    let primary = phone_repo
        .find_primary(&user.id)
        .await?
        .expect("primary phone number");
    assert_eq!(primary.id, second.id);
    assert!(primary.is_verified);

    Ok(())
}
