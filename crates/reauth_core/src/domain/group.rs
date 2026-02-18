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
