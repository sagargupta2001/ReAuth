use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
}

// Manual implementation to safely map SQLite Strings -> Rust Uuid
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Role {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // Helper closure to parse a UUID string from a column
        let parse_uuid = |val: String, col_name: &str| -> Result<Uuid, sqlx::Error> {
            Uuid::parse_str(&val).map_err(|e| sqlx::Error::ColumnDecode {
                index: col_name.into(),
                source: Box::new(e),
            })
        };

        // Get raw strings from the database
        let id_str: String = row.try_get("id")?;
        let realm_id_str: String = row.try_get("realm_id")?;

        // Handle the Nullable Client ID
        let client_id_str: Option<String> = row.try_get("client_id")?;
        let client_id = match client_id_str {
            Some(s) => Some(parse_uuid(s, "client_id")?),
            None => None,
        };

        Ok(Role {
            id: parse_uuid(id_str, "id")?,
            realm_id: parse_uuid(realm_id_str, "realm_id")?,
            client_id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
        })
    }
}

pub type Permission = String;

#[cfg(test)]
mod tests {
    use super::*;
    // use super::Role;
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
}
