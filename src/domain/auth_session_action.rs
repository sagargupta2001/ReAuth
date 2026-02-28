use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthSessionAction {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub session_id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub action_type: String,
    pub token_hash: String,
    #[sqlx(rename = "payload_json")]
    #[sqlx(json)]
    pub payload: Value,
    pub resume_node_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthSessionAction {
    pub fn new(
        session_id: Uuid,
        realm_id: Uuid,
        action_type: String,
        token_hash: String,
        payload: Value,
        resume_node_id: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            session_id,
            realm_id,
            action_type,
            token_hash,
            payload,
            resume_node_id,
            expires_at,
            consumed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed_at.is_some()
    }
}
