use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    #[sqlx(try_from = "String")] // Convert TEXT from DB to Uuid
    pub id: Uuid,
    pub username: String,
    pub hashed_password: String,
}
