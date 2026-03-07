use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportPolicy {
    Redact,
    IncludeSecrets,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarborExportType {
    Theme,
    Client,
    Flow,
    Role,
    FullRealm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    #[default]
    Skip,
    Overwrite,
    Rename,
}

#[derive(Debug, Clone)]
pub enum HarborScope {
    Theme { theme_id: Uuid },
    Client { client_id: String },
    Flow { flow_id: Uuid },
    Role { role_id: Uuid },
    FullRealm,
}

impl HarborScope {
    pub fn export_type(&self) -> HarborExportType {
        match self {
            HarborScope::Theme { .. } => HarborExportType::Theme,
            HarborScope::Client { .. } => HarborExportType::Client,
            HarborScope::Flow { .. } => HarborExportType::Flow,
            HarborScope::Role { .. } => HarborExportType::Role,
            HarborScope::FullRealm => HarborExportType::FullRealm,
        }
    }

    pub fn provider_key(&self) -> Option<&'static str> {
        match self {
            HarborScope::Theme { .. } => Some("theme"),
            HarborScope::Client { .. } => Some("client"),
            HarborScope::Flow { .. } => Some("flow"),
            HarborScope::Role { .. } => Some("role"),
            HarborScope::FullRealm => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborManifest {
    pub version: String,
    pub schema_version: u32,
    pub exported_at: String,
    pub source_realm: String,
    #[serde(rename = "type")]
    pub export_type: HarborExportType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborAsset {
    pub id: Option<String>,
    pub filename: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborResourceBundle {
    pub key: String,
    pub data: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<HarborAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborBundle {
    pub manifest: HarborManifest,
    pub resources: Vec<HarborResourceBundle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborImportResourceResult {
    pub key: String,
    pub status: String,
    pub created: u32,
    pub updated: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renamed_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborImportResult {
    pub dry_run: bool,
    pub resources: Vec<HarborImportResourceResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarborThemeMeta {
    #[serde(default = "default_true")]
    pub draft_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<HarborThemeMetaTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bindings: Option<HarborThemeBindings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborThemeMetaTheme {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarborThemeBindings {
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub clients: Vec<HarborThemeClientBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborThemeClientBinding {
    pub client_id: String,
}
