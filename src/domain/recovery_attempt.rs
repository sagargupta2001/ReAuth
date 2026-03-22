use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    pub realm_id: Uuid,
    pub identifier: String,
    pub window_started_at: DateTime<Utc>,
    pub attempt_count: i64,
    pub updated_at: DateTime<Utc>,
}
