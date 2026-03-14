use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmRecoverySettings {
    pub realm_id: Uuid,
    pub token_ttl_minutes: i64,
    pub rate_limit_max: i64,
    pub rate_limit_window_minutes: i64,
    pub revoke_sessions_on_reset: bool,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
}

impl RealmRecoverySettings {
    pub fn defaults(realm_id: Uuid) -> Self {
        Self {
            realm_id,
            token_ttl_minutes: 15,
            rate_limit_max: 5,
            rate_limit_window_minutes: 15,
            revoke_sessions_on_reset: true,
            email_subject: None,
            email_body: None,
        }
    }
}
