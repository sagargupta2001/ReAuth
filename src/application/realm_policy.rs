use serde::Serialize;
use uuid::Uuid;

use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::realm::Realm;

#[derive(Debug, Clone, Serialize)]
pub struct RealmCapabilities {
    pub registration_enabled: bool,
    pub default_registration_role_ids: Vec<Uuid>,
}

impl RealmCapabilities {
    pub fn from_realm(realm: &Realm) -> Self {
        let is_system = realm.is_system || realm.name == DEFAULT_REALM_NAME;
        Self {
            registration_enabled: !is_system && realm.registration_enabled,
            default_registration_role_ids: realm.default_registration_role_ids.clone(),
        }
    }
}

pub fn registration_enabled(realm: &Realm) -> bool {
    RealmCapabilities::from_realm(realm).registration_enabled
}
