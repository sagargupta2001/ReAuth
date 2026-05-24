use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OAuthStartAttempt {
    pub realm_id: Uuid,
    pub provider_id: Uuid,
    pub ip_address: String,
    pub window_started_at: DateTime<Utc>,
    pub attempt_count: i64,
    pub updated_at: DateTime<Utc>,
}
