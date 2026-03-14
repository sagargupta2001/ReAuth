use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmEmailSettings {
    pub realm_id: Uuid,
    pub enabled: bool,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_security: String,
}

impl RealmEmailSettings {
    pub fn disabled(realm_id: Uuid) -> Self {
        Self {
            realm_id,
            enabled: false,
            from_address: None,
            from_name: None,
            reply_to_address: None,
            smtp_host: None,
            smtp_port: None,
            smtp_username: None,
            smtp_password: None,
            smtp_security: "starttls".to_string(),
        }
    }
}
