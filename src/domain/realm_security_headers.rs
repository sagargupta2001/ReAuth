use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmSecurityHeaders {
    pub realm_id: Uuid,
    pub x_frame_options: Option<String>,
    pub content_security_policy: Option<String>,
    pub x_content_type_options: Option<String>,
    pub referrer_policy: Option<String>,
    pub strict_transport_security: Option<String>,
}

impl RealmSecurityHeaders {
    pub fn defaults(realm_id: Uuid) -> Self {
        Self {
            realm_id,
            x_frame_options: Some("SAMEORIGIN".to_string()),
            content_security_policy: Some("frame-ancestors 'self'".to_string()),
            x_content_type_options: Some("nosniff".to_string()),
            referrer_policy: Some("no-referrer".to_string()),
            strict_transport_security: None,
        }
    }
}
