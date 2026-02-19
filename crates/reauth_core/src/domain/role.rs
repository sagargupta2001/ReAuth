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
mod role_tests;
