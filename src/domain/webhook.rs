use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub status: String,
    pub signing_secret: String,
    pub custom_headers: HashMap<String, String>,
    pub description: Option<String>,
    pub consecutive_failures: i64,
    pub last_fired_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub disabled_at: Option<String>,
    pub disabled_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub endpoint_id: Uuid,
    pub event_type: String,
    pub enabled: bool,
    pub created_at: String,
}
