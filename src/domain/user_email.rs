use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserEmail {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub user_id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub email: String,
    pub email_normalized: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserEmail {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        email: String,
        is_primary: bool,
        is_verified: bool,
    ) -> Self {
        let email_normalized = email.trim().to_lowercase();
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            email: email.trim().to_string(),
            email_normalized,
            is_primary,
            is_verified,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
