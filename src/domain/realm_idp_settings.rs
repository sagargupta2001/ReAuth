use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmIdpSettings {
    pub realm_id: Uuid,
    pub oauth_start_rate_limit_max: i64,
    pub oauth_start_rate_limit_window_minutes: i64,
}

impl RealmIdpSettings {
    pub fn defaults(realm_id: Uuid) -> Self {
        Self {
            realm_id,
            oauth_start_rate_limit_max: 30,
            oauth_start_rate_limit_window_minutes: 10,
        }
    }
}
