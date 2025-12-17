use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Group {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}
