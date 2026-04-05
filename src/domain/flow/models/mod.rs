use crate::domain::ui::{PageCategory, UiSurface};
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
    pub id: String,
    pub flow_id: String,
    pub version_number: i32,
    // This holds the linearized instruction set for the engine
    pub execution_artifact: String,
    pub graph_json: String,
    pub checksum: String,
    #[serde(default = "default_node_contract_versions")]
    pub node_contract_versions: String,
    pub created_at: DateTime<Utc>,
}

fn default_node_contract_versions() -> String {
    "{}".to_string()
}

// --- DEPLOYMENT (Active Pointer) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowDeployment {
    pub id: String,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub flow_type: String, // e.g. "browser", "api", "registration"
    #[sqlx(try_from = "String")]
    pub active_version_id: String,
    pub updated_at: DateTime<Utc>,
}

// --- NODE REGISTRY ---
// These structs define what nodes are AVAILABLE in the palette.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeCapabilities {
    #[serde(default)]
    pub supports_ui: bool,
    #[serde(default)]
    pub ui_surface: Option<UiSurface>,
    #[serde(default)]
    pub allowed_page_categories: Vec<PageCategory>,
    #[serde(default)]
    pub async_pause: bool,
    #[serde(default)]
    pub side_effects: bool,
    #[serde(default)]
    pub requires_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContract {
    pub id: String,       // e.g., "core.auth.password"
    pub category: String, // "Authenticator", "Condition", "Action"
    pub display_name: String,
    pub description: String,
    pub icon: String,                     // Icon name for the UI
    pub config_schema: serde_json::Value, // JSON Schema for the config form
    pub inputs: Vec<String>,              // e.g., ["flow"]
    pub outputs: Vec<String>,             // e.g., ["success", "failure"]
    #[serde(default)]
    pub default_template_key: Option<String>,
    #[serde(default)]
    pub contract_version: String,
    #[serde(default)]
    pub capabilities: NodeCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowPublishIssue {
    pub message: String,
    #[serde(default)]
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowPublishValidation {
    pub message: String,
    #[serde(default)]
    pub issues: Vec<FlowPublishIssue>,
}

impl std::fmt::Display for FlowPublishValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests;
