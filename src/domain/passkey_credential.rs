use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub user_id: Uuid,
    pub credential_id_b64url: String,
    pub public_key_cose_b64url: String,
    pub sign_count: i64,
    pub transports_json: String,
    pub backed_up: bool,
    pub backup_eligible: bool,
    pub aaguid: Option<String>,
    pub friendly_name: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PasskeyCredential {
    pub fn new(
        realm_id: Uuid,
        user_id: Uuid,
        credential_id_b64url: String,
        public_key_cose_b64url: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            realm_id,
            user_id,
            credential_id_b64url,
            public_key_cose_b64url,
            sign_count: 0,
            transports_json: "[]".to_string(),
            backed_up: false,
            backup_eligible: false,
            aaguid: None,
            friendly_name: None,
            last_used_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}
