use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Theme {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeTokens {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub tokens_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeLayout {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub name: String,
    pub layout_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeNode {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub node_key: String,
    pub blueprint_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeAsset {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub asset_type: String,
    pub filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub checksum: Option<String>,
    pub data: Vec<u8>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeAssetMeta {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub asset_type: String,
    pub filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub checksum: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeVersion {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    pub version_number: i64,
    pub status: String,
    pub snapshot_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThemeBinding {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub client_id: Option<String>,
    #[sqlx(try_from = "String")]
    pub theme_id: Uuid,
    #[sqlx(try_from = "String")]
    pub active_version_id: Uuid,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeNodeInstance {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub props: serde_json::Value,
    #[serde(default)]
    pub layout: Option<serde_json::Value>,
    #[serde(default)]
    pub size: Option<serde_json::Value>,
    #[serde(default)]
    pub children: Vec<ThemeNodeInstance>,
    #[serde(default)]
    pub slots: std::collections::HashMap<String, ThemeNodeInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAssetRef {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub url: String,
    pub checksum: Option<String>,
    pub byte_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSnapshot {
    pub theme_id: Uuid,
    pub version_id: Uuid,
    pub tokens: serde_json::Value,
    pub layout: serde_json::Value,
    pub nodes: Vec<ThemeNodeInstance>,
    pub assets: Vec<ThemeAssetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDraftNode {
    pub node_key: String,
    pub blueprint: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDraft {
    pub tokens: serde_json::Value,
    pub layout: serde_json::Value,
    pub nodes: Vec<ThemeDraftNode>,
}
