use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoginAttempt {
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub username: String,
    pub failed_count: i64,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
