use super::Role;
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn role_from_row_parses_null_client_id() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let role: Role = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, NULL as client_id, ? as name, ? as description",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind("admin")
    .bind("Admin role")
    .fetch_one(&pool)
    .await
    .expect("fetch role");

    assert_eq!(role.id, id);
    assert_eq!(role.realm_id, realm_id);
    assert!(role.client_id.is_none());
    assert_eq!(role.name, "admin");
    assert_eq!(role.description.as_deref(), Some("Admin role"));
}

#[tokio::test]
async fn role_from_row_rejects_invalid_client_id() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let result: Result<Role, sqlx::Error> = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as client_id, ? as name, ? as description",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind("not-a-uuid")
    .bind("admin")
    .bind("Admin role")
    .fetch_one(&pool)
    .await;

    assert!(result.is_err());
}
