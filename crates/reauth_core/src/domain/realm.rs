use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Realm {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub name: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
}
