use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub user_id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub client_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        client_id: Option<String>,
        expires_in: chrono::Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            client_id,
            expires_at: now + expires_in,
            ip_address: None,
            user_agent: None,
            created_at: now,
            last_used_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

#[cfg(test)]
mod session_tests;
