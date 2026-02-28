use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct AuditEvent {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct NewAuditEvent {
    pub realm_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub metadata: Value,
}
