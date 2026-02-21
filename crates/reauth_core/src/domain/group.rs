use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
}

// Manual implementation to safely map SQLite Strings -> Rust Uuid (including optional parent_id)
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Group {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let parse_uuid = |val: String, col_name: &str| -> Result<Uuid, sqlx::Error> {
            Uuid::parse_str(&val).map_err(|e| sqlx::Error::ColumnDecode {
                index: col_name.into(),
                source: Box::new(e),
            })
        };

        let id_str: String = row.try_get("id")?;
        let realm_id_str: String = row.try_get("realm_id")?;
        let parent_id_str: Option<String> = row.try_get("parent_id")?;
        let parent_id = match parent_id_str {
            Some(s) => Some(parse_uuid(s, "parent_id")?),
            None => None,
        };

        Ok(Group {
            id: parse_uuid(id_str, "id")?,
            realm_id: parse_uuid(realm_id_str, "realm_id")?,
            parent_id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            sort_order: row.try_get("sort_order")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::Group;
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
}
