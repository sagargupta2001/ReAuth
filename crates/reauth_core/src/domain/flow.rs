use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// --- DRAFT (Editable) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowDraft {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    // This holds the React Flow nodes/edges array
    pub graph_json: String,
    #[sqlx(try_from = "String")]
    pub flow_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- VERSION (Immutable) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowVersion {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub draft_id: Uuid,
    pub version_number: i64,
    // This holds the linearized instruction set for the engine
    pub execution_artifact: String,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

// --- DEPLOYMENT (Active Pointer) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowDeployment {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub flow_type: String, // e.g. "browser", "api", "registration"
    #[sqlx(try_from = "String")]
    pub active_version_id: Uuid,
    pub updated_at: DateTime<Utc>,
}

// --- NODE REGISTRY ---
// These structs define what nodes are AVAILABLE in the palette.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: String,       // e.g., "core.auth.password"
    pub category: String, // "Authenticator", "Condition", "Action"
    pub display_name: String,
    pub description: String,
    pub icon: String,                     // Icon name for the UI
    pub config_schema: serde_json::Value, // JSON Schema for the config form
    pub inputs: Vec<String>,              // e.g., ["flow"]
    pub outputs: Vec<String>,             // e.g., ["success", "failure"]
}
