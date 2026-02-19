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

impl User {
    pub fn new(realm_id: Uuid, username: String, hashed_password: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id,
            username,
            hashed_password,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod user_tests;
