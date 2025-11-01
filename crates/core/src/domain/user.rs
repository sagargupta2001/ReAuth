use serde::{Deserialize, Serialize};

/// Represents a User entity within the application's domain.
/// This is pure data and has no knowledge of how it is stored or presented.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: String,
}