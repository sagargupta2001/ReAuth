use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmPasskeySettings {
    pub realm_id: Uuid,
    pub enabled: bool,
    pub allow_password_fallback: bool,
    pub discoverable_preferred: bool,
    pub challenge_ttl_secs: i64,
    pub reauth_max_age_secs: i64,
}

impl RealmPasskeySettings {
    pub fn defaults(realm_id: Uuid) -> Self {
        Self {
            realm_id,
            enabled: false,
            allow_password_fallback: true,
            discoverable_preferred: true,
            challenge_ttl_secs: 120,
            reauth_max_age_secs: 300,
        }
    }
}
