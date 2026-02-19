use super::User;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn user_from_row_works() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let user: User =
        sqlx::query_as("SELECT ? as id, ? as realm_id, ? as username, ? as hashed_password")
            .bind(id.to_string())
            .bind(realm_id.to_string())
            .bind("alice")
            .bind("hash")
            .fetch_one(&pool)
            .await
            .expect("fetch user");

    assert_eq!(user.id, id);
    assert_eq!(user.realm_id, realm_id);
    assert_eq!(user.username, "alice");
    assert_eq!(user.hashed_password, "hash");
}

#[test]
fn user_serialization_skips_hashed_password() {
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "alice".to_string(),
        hashed_password: "hash".to_string(),
    };

    let value = serde_json::to_value(&user).expect("serialize");
    assert!(value.get("hashed_password").is_none());
}

#[test]
fn user_deserializes_with_hashed_password() {
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let value = json!({
        "id": id,
        "realm_id": realm_id,
        "username": "alice",
        "hashed_password": "hash"
    });

    let user: User = serde_json::from_value(value).expect("deserialize");

    assert_eq!(user.id, id);
    assert_eq!(user.realm_id, realm_id);
    assert_eq!(user.username, "alice");
    assert_eq!(user.hashed_password, "hash");
}

#[test]
fn user_new_generates_id() {
    let realm_id = Uuid::new_v4();
    let user = User::new(realm_id, "bob".to_string(), "hash".to_string());

    assert!(!user.id.is_nil());
    assert_eq!(user.realm_id, realm_id);
    assert_eq!(user.username, "bob");
}

#[test]
fn user_validation_works() {
    let realm_id = Uuid::new_v4();

    let valid_user = User::new(realm_id, "alice".to_string(), "hash".to_string());
    assert!(valid_user.validate().is_ok());

    let empty_user = User::new(realm_id, "".to_string(), "hash".to_string());
    assert!(empty_user.validate().is_err());

    let short_user = User::new(realm_id, "al".to_string(), "hash".to_string());
    assert!(short_user.validate().is_err());
}
