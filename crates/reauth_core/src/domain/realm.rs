use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Realm {
    #[sqlx(try_from = "String")]
    pub id: Uuid, // Mandatory field works fine with try_from
    pub name: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,

    // This matches the SQLite TEXT column perfectly.
    pub browser_flow_id: Option<String>,
    pub registration_flow_id: Option<String>,
    pub direct_grant_flow_id: Option<String>,
    pub reset_credentials_flow_id: Option<String>,
    // --------------------------------------------------
}

impl Realm {
    // Optional helper if you need Uuids in your code logic
    pub fn browser_flow_uuid(&self) -> Option<Uuid> {
        self.browser_flow_id
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok())
    }
}

#[cfg(test)]
mod realm_tests;
