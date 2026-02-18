use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    #[sqlx(try_from = "String")] // Convert TEXT from DB to Uuid
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub username: String,
    #[serde(skip_serializing)] // Don't send hash to UI
    pub hashed_password: String,
}

#[cfg(test)]
mod user_tests;
