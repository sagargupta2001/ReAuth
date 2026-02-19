use super::{CustomPermission, GroupTreeRow};
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn group_tree_row_from_row_parses_fields() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    let row: GroupTreeRow = sqlx::query_as(
        "SELECT ? as id, ? as parent_id, ? as name, ? as description, ? as sort_order, ? as has_children",
    )
    .bind(id.to_string())
    .bind(parent_id.to_string())
    .bind("group")
    .bind("Group row")
    .bind(4_i64)
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("fetch group tree row");

    assert_eq!(row.id, id);
    assert_eq!(row.parent_id, Some(parent_id));
    assert_eq!(row.name, "group");
    assert_eq!(row.description.as_deref(), Some("Group row"));
    assert_eq!(row.sort_order, 4);
    assert!(row.has_children);
}

#[tokio::test]
async fn custom_permission_from_row_parses_optional_fields() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let row: CustomPermission = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, NULL as client_id, ? as permission, ? as name, NULL as description, NULL as created_by",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind("perm.read")
    .bind("Read")
    .fetch_one(&pool)
    .await
    .expect("fetch custom permission");

    assert_eq!(row.id, id);
    assert_eq!(row.realm_id, realm_id);
    assert!(row.client_id.is_none());
    assert_eq!(row.permission, "perm.read");
    assert_eq!(row.name, "Read");
    assert!(row.description.is_none());
    assert!(row.created_by.is_none());
}

#[tokio::test]
async fn custom_permission_from_row_rejects_invalid_uuid() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let realm_id = Uuid::new_v4();

    let result: Result<CustomPermission, sqlx::Error> = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, NULL as client_id, ? as permission, ? as name, NULL as description, NULL as created_by",
    )
    .bind("not-a-uuid")
    .bind(realm_id.to_string())
    .bind("perm.read")
    .bind("Read")
    .fetch_one(&pool)
    .await;

    assert!(result.is_err());
}
