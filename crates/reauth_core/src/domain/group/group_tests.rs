use super::Group;
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn group_from_row_parses_parent_id_none() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let group: Group = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, NULL as parent_id, ? as name, ? as description, ? as sort_order",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind("root")
    .bind("Root group")
    .bind(0_i64)
    .fetch_one(&pool)
    .await
    .expect("fetch group");

    assert_eq!(group.id, id);
    assert_eq!(group.realm_id, realm_id);
    assert!(group.parent_id.is_none());
    assert_eq!(group.name, "root");
    assert_eq!(group.description.as_deref(), Some("Root group"));
    assert_eq!(group.sort_order, 0);
}

#[tokio::test]
async fn group_from_row_parses_parent_id_some() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    let group: Group = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as parent_id, ? as name, ? as description, ? as sort_order",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind(parent_id.to_string())
    .bind("child")
    .bind("Child group")
    .bind(1_i64)
    .fetch_one(&pool)
    .await
    .expect("fetch group");

    assert_eq!(group.parent_id, Some(parent_id));
    assert_eq!(group.sort_order, 1);
}

#[tokio::test]
async fn group_from_row_rejects_invalid_parent_id() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let result: Result<Group, sqlx::Error> = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as parent_id, ? as name, ? as description, ? as sort_order",
    )
    .bind(id.to_string())
    .bind(realm_id.to_string())
    .bind("bad-parent")
    .bind("child")
    .bind("Child group")
    .bind(1_i64)
    .fetch_one(&pool)
    .await;

    assert!(result.is_err());
}
