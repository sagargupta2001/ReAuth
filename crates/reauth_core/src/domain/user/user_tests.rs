use super::User;
use serde_json::json;
use uuid::Uuid;

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
