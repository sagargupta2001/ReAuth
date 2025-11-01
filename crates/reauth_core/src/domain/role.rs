use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Permission = String;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Role {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}