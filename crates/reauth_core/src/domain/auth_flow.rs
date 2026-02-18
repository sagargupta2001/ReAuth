use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a configured flow (e.g., "browser-login")
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuthFlow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub alias: String,
    pub r#type: String, // "type" is a reserved keyword
    pub built_in: bool,
}

#[cfg(test)]
mod auth_flow_tests;
