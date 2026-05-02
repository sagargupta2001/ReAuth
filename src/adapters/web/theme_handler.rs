use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::theme::ThemeDraft;
use crate::error::{Error, Result};
use crate::AppState;

#[derive(Deserialize)]
pub struct ThemeResolveParams {
    pub client_id: Option<String>,
    pub node_key: Option<String>,
    pub page_key: Option<String>,
}

#[derive(Deserialize)]
pub struct ThemePreviewParams {
    pub node_key: Option<String>,
    pub page_key: Option<String>,
}

#[derive(Serialize)]
pub struct ThemeSummary {
    pub id: String,
    pub realm_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ThemeDetailsResponse {
    pub theme: ThemeSummary,
    pub active_version_id: Option<String>,
    pub active_version_number: Option<i64>,
}

#[derive(Serialize)]
pub struct ActiveThemeResponse {
    pub theme: ThemeSummary,
    pub active_version_id: Option<String>,
    pub active_version_number: Option<i64>,
    pub pages: Vec<crate::domain::theme_pages::ThemePageTemplate>,
}

#[derive(Serialize)]
pub struct ThemeVersionSummary {
    pub id: String,
    pub theme_id: String,
    pub version_number: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ThemeVersionSnapshotResponse {
    pub version_id: String,
    pub theme_id: String,
    pub version_number: i64,
    pub snapshot: ThemeDraft,
}

#[derive(Deserialize)]
pub struct ThemeBindingPayload {
    pub version_id: String,
}

#[derive(Serialize)]
pub struct ThemeBindingSummary {
    pub client_id: String,
    pub theme_id: String,
    pub active_version_id: String,
    pub active_version_number: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ThemeBundleAsset {
    pub id: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub asset_type: String,
    pub data_base64: String,
}

#[derive(Serialize, Deserialize)]
pub struct ThemeBundlePayload {
    pub tokens: Value,
    pub layout: Value,
    pub nodes: Vec<crate::domain::theme::ThemeDraftNode>,
    pub assets: Vec<ThemeBundleAsset>,
}

#[derive(Serialize)]
pub struct ThemeAssetSummary {
    pub id: String,
    pub theme_id: String,
    pub asset_type: String,
    pub filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub checksum: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct ThemeTemplateGapResponse {
    pub missing: Vec<String>,
}

#[derive(Deserialize)]
pub struct ThemePagesQuery {
    pub theme_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateThemeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateThemeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn resolve_theme_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(params): Query<ThemeResolveParams>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let snapshot = state
        .theme_service
        .resolve_snapshot(
            realm.id,
            &realm.name,
            params.client_id.as_deref(),
            params.page_key.as_deref().or(params.node_key.as_deref()),
        )
        .await?;

    Ok((StatusCode::OK, Json(snapshot)))
}

pub async fn create_theme_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateThemeRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme = state
        .theme_service
        .create_theme(realm.id, payload.name, payload.description)
        .await?;

    let active_version_id = state
        .theme_service
        .get_active_version_id(realm.id, &theme.id)
        .await?
        .map(|id| id.to_string());
    let active_version_number = state
        .theme_service
        .get_active_version_number(realm.id, &theme.id)
        .await?;

    let response = ThemeDetailsResponse {
        theme: ThemeSummary {
            id: theme.id.to_string(),
            realm_id: theme.realm_id.to_string(),
            name: theme.name,
            description: theme.description,
            is_system: theme.is_system,
            created_at: theme.created_at,
            updated_at: theme.updated_at,
        },
        active_version_id,
        active_version_number,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_theme_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
    Json(payload): Json<UpdateThemeRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let theme = state
        .theme_service
        .update_theme(realm.id, theme_uuid, payload.name, payload.description)
        .await?;

    let active_version_id = state
        .theme_service
        .get_active_version_id(realm.id, &theme.id)
        .await?
        .map(|id| id.to_string());
    let active_version_number = state
        .theme_service
        .get_active_version_number(realm.id, &theme.id)
        .await?;

    let response = ThemeDetailsResponse {
        theme: ThemeSummary {
            id: theme.id.to_string(),
            realm_id: theme.realm_id.to_string(),
            name: theme.name,
            description: theme.description,
            is_system: theme.is_system,
            created_at: theme.created_at,
            updated_at: theme.updated_at,
        },
        active_version_id,
        active_version_number,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_themes_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let themes = state.theme_service.list_themes(realm.id).await?;
    let response: Vec<ThemeSummary> = themes
        .into_iter()
        .map(|theme| ThemeSummary {
            id: theme.id.to_string(),
            realm_id: theme.realm_id.to_string(),
            name: theme.name,
            description: theme.description,
            is_system: theme.is_system,
            created_at: theme.created_at,
            updated_at: theme.updated_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_theme_pages_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<ThemePagesQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let pages = if let Some(theme_id) = query.theme_id {
        state
            .theme_service
            .list_pages_for_theme(realm.id, theme_id)
            .await?
    } else {
        state.theme_service.list_pages()
    };
    Ok((StatusCode::OK, Json(pages)))
}

pub async fn get_active_theme_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let binding = state
        .theme_service
        .resolve_binding(realm.id, None)
        .await?
        .ok_or_else(|| Error::NotFound("Active theme not found".to_string()))?;

    let theme = state
        .theme_service
        .get_theme(realm.id, &binding.theme_id)
        .await?
        .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

    let pages = state
        .theme_service
        .list_pages_for_theme(realm.id, binding.theme_id)
        .await?;

    let response = ActiveThemeResponse {
        theme: ThemeSummary {
            id: theme.id.to_string(),
            realm_id: theme.realm_id.to_string(),
            name: theme.name,
            description: theme.description,
            is_system: theme.is_system,
            created_at: theme.created_at,
            updated_at: theme.updated_at,
        },
        active_version_id: Some(binding.active_version_id.to_string()),
        active_version_number: state
            .theme_service
            .get_active_version_number(realm.id, &binding.theme_id)
            .await?,
        pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_theme_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let theme = state
        .theme_service
        .get_theme(realm.id, &theme_uuid)
        .await?
        .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

    let active_version_id = state
        .theme_service
        .get_active_version_id(realm.id, &theme_uuid)
        .await?
        .map(|id| id.to_string());
    let active_version_number = state
        .theme_service
        .get_active_version_number(realm.id, &theme_uuid)
        .await?;

    let response = ThemeDetailsResponse {
        theme: ThemeSummary {
            id: theme.id.to_string(),
            realm_id: theme.realm_id.to_string(),
            name: theme.name,
            description: theme.description,
            is_system: theme.is_system,
            created_at: theme.created_at,
            updated_at: theme.updated_at,
        },
        active_version_id,
        active_version_number,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn preview_theme_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
    Query(params): Query<ThemePreviewParams>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let snapshot = state
        .theme_service
        .preview_snapshot(
            realm.id,
            &realm.name,
            theme_uuid,
            params.page_key.as_deref().or(params.node_key.as_deref()),
        )
        .await?;

    Ok((StatusCode::OK, Json(snapshot)))
}

pub async fn list_theme_template_gaps_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let pages = state
        .theme_service
        .list_pages_for_theme(realm.id, theme_uuid)
        .await?;
    let page_keys: HashSet<String> = pages.into_iter().map(|page| page.key).collect();

    let drafts = state.flow_manager.list_all_drafts(realm.id).await?;
    let mut used_templates: HashSet<String> = HashSet::new();

    for draft in drafts {
        let graph: Value = match serde_json::from_str(&draft.graph_json) {
            Ok(value) => value,
            Err(_) => continue,
        };
        for key in extract_template_keys(&graph) {
            used_templates.insert(key);
        }
    }

    let mut missing: Vec<String> = used_templates
        .into_iter()
        .filter(|key| !page_keys.contains(key))
        .collect();
    missing.sort();

    Ok((StatusCode::OK, Json(ThemeTemplateGapResponse { missing })))
}

pub async fn list_theme_bindings_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let theme = state
        .theme_service
        .get_theme(realm.id, &theme_uuid)
        .await?
        .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

    let bindings = state
        .theme_service
        .list_bindings_for_theme(realm.id, theme.id)
        .await?;

    let mut response = Vec::new();
    for binding in bindings {
        let version_number = state
            .theme_service
            .get_version_meta(&theme.id, &binding.active_version_id)
            .await?
            .map(|version| version.version_number);

        response.push(ThemeBindingSummary {
            client_id: binding.client_id.unwrap_or_default(),
            theme_id: binding.theme_id.to_string(),
            active_version_id: binding.active_version_id.to_string(),
            active_version_number: version_number,
        });
    }

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_theme_binding_handler(
    State(state): State<AppState>,
    Path((realm_name, client_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    if client_id.trim().is_empty() {
        return Err(Error::Validation("Client id is required".to_string()));
    }

    let binding = state
        .theme_service
        .get_binding_for_client(realm.id, &client_id)
        .await?;

    let response = if let Some(binding) = binding {
        let version_number = state
            .theme_service
            .get_version_meta(&binding.theme_id, &binding.active_version_id)
            .await?
            .map(|version| version.version_number);

        Some(ThemeBindingSummary {
            client_id: binding.client_id.unwrap_or_default(),
            theme_id: binding.theme_id.to_string(),
            active_version_id: binding.active_version_id.to_string(),
            active_version_number: version_number,
        })
    } else {
        None
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn upsert_theme_binding_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id, client_id)): Path<(String, String, String)>,
    Json(payload): Json<ThemeBindingPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
    let version_uuid = Uuid::parse_str(&payload.version_id)
        .map_err(|_| Error::Validation("Invalid version id".to_string()))?;

    if client_id.trim().is_empty() {
        return Err(Error::Validation("Client id is required".to_string()));
    }

    let binding = state
        .theme_service
        .upsert_client_binding(realm.id, client_id, theme_uuid, version_uuid)
        .await?;

    let version_number = state
        .theme_service
        .get_version_meta(&binding.theme_id, &binding.active_version_id)
        .await?
        .map(|version| version.version_number);

    Ok((
        StatusCode::OK,
        Json(ThemeBindingSummary {
            client_id: binding.client_id.unwrap_or_default(),
            theme_id: binding.theme_id.to_string(),
            active_version_id: binding.active_version_id.to_string(),
            active_version_number: version_number,
        }),
    ))
}

pub async fn delete_theme_binding_handler(
    State(state): State<AppState>,
    Path((realm_name, _theme_id, client_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    if client_id.trim().is_empty() {
        return Err(Error::Validation("Client id is required".to_string()));
    }

    state
        .theme_service
        .delete_client_binding(realm.id, &client_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn export_theme_bundle_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let draft = state.theme_service.get_draft(realm.id, theme_uuid).await?;
    let assets_meta = state
        .theme_service
        .list_assets(realm.id, theme_uuid)
        .await?;
    let mut assets = Vec::new();

    for meta in assets_meta {
        let asset = state
            .theme_service
            .get_asset(&theme_uuid, &meta.id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme asset not found".to_string()))?;
        assets.push(ThemeBundleAsset {
            id: Some(asset.id.to_string()),
            filename: asset.filename,
            mime_type: asset.mime_type,
            asset_type: asset.asset_type,
            data_base64: STANDARD.encode(asset.data),
        });
    }

    Ok((
        StatusCode::OK,
        Json(ThemeBundlePayload {
            tokens: draft.tokens,
            layout: draft.layout,
            nodes: draft.nodes,
            assets,
        }),
    ))
}

pub async fn import_theme_bundle_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
    Json(payload): Json<ThemeBundlePayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let mut id_map: HashMap<String, String> = HashMap::new();
    for asset in payload.assets {
        if asset.data_base64.trim().is_empty() {
            continue;
        }
        let data = STANDARD
            .decode(asset.data_base64.as_bytes())
            .map_err(|_| Error::Validation("Invalid asset data".to_string()))?;
        let created = state
            .theme_service
            .create_asset(
                realm.id,
                theme_uuid,
                asset.asset_type,
                asset.filename,
                asset.mime_type,
                data,
            )
            .await?;
        if let Some(old_id) = asset.id {
            id_map.insert(old_id, created.id.to_string());
        }
    }

    let mut draft = crate::domain::theme::ThemeDraft {
        tokens: payload.tokens,
        layout: payload.layout,
        nodes: payload.nodes,
    };

    for node in draft.nodes.iter_mut() {
        rewrite_blueprint_assets(&mut node.blueprint, &id_map);
    }

    state
        .theme_service
        .save_draft(realm.id, theme_uuid, draft)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

fn rewrite_blueprint_assets(value: &mut Value, id_map: &HashMap<String, String>) {
    if let Some(obj) = value.as_object_mut() {
        if let Some(props) = obj.get_mut("props").and_then(|value| value.as_object_mut()) {
            if let Some(asset_id) = props.get_mut("asset_id") {
                if let Some(asset_str) = asset_id.as_str() {
                    if let Some(replacement) = id_map.get(asset_str) {
                        *asset_id = Value::String(replacement.clone());
                    }
                }
            }
        }
        if let Some(children) = obj
            .get_mut("children")
            .and_then(|value| value.as_array_mut())
        {
            for child in children {
                rewrite_blueprint_assets(child, id_map);
            }
        }
        if let Some(slots) = obj.get_mut("slots").and_then(|value| value.as_object_mut()) {
            for slot in slots.values_mut() {
                rewrite_blueprint_assets(slot, id_map);
            }
        }
        if let Some(nodes) = obj.get_mut("nodes").and_then(|value| value.as_array_mut()) {
            for node in nodes {
                rewrite_blueprint_assets(node, id_map);
            }
        }
    } else if let Some(arr) = value.as_array_mut() {
        for item in arr {
            rewrite_blueprint_assets(item, id_map);
        }
    }
}

pub async fn list_theme_assets_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let assets = state
        .theme_service
        .list_assets(realm.id, theme_uuid)
        .await?;

    let response: Vec<ThemeAssetSummary> = assets
        .into_iter()
        .map(|asset| ThemeAssetSummary {
            id: asset.id.to_string(),
            theme_id: asset.theme_id.to_string(),
            asset_type: asset.asset_type,
            filename: asset.filename,
            mime_type: asset.mime_type,
            byte_size: asset.byte_size,
            checksum: asset.checksum,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
            url: format!(
                "/api/realms/{}/theme/{}/assets/{}",
                realm.name, theme_id, asset.id
            ),
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn upload_theme_asset_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    const MAX_ASSET_BYTES: usize = 5 * 1024 * 1024;

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let mut asset_type: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| Error::Validation(format!("Invalid multipart data: {}", err)))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "asset_type" {
            let value = field
                .text()
                .await
                .map_err(|err| Error::Validation(format!("Invalid asset_type: {}", err)))?;
            let trimmed = value.trim().to_string();
            if !trimmed.is_empty() {
                asset_type = Some(trimmed);
            }
        } else if name == "file" {
            filename = field.file_name().map(|name| name.to_string());
            mime_type = field.content_type().map(|ct| ct.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|err| Error::Validation(format!("Invalid file data: {}", err)))?;
            if bytes.len() > MAX_ASSET_BYTES {
                return Err(Error::Validation(format!(
                    "Asset exceeds {} MB limit",
                    MAX_ASSET_BYTES / (1024 * 1024)
                )));
            }
            data = Some(bytes.to_vec());
        }
    }

    let data = data.ok_or_else(|| Error::Validation("File is required".to_string()))?;
    if data.is_empty() {
        return Err(Error::Validation("Uploaded file is empty".to_string()));
    }

    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let filename = filename.unwrap_or_else(|| "asset".to_string());
    let asset_type = asset_type.unwrap_or_else(|| {
        if mime_type.starts_with("image/") {
            "image".to_string()
        } else if mime_type.starts_with("font/") {
            "font".to_string()
        } else {
            "file".to_string()
        }
    });

    let asset = state
        .theme_service
        .create_asset(realm.id, theme_uuid, asset_type, filename, mime_type, data)
        .await?;

    let response = ThemeAssetSummary {
        id: asset.id.to_string(),
        theme_id: asset.theme_id.to_string(),
        asset_type: asset.asset_type,
        filename: asset.filename,
        mime_type: asset.mime_type,
        byte_size: asset.byte_size,
        checksum: asset.checksum,
        created_at: asset.created_at,
        updated_at: asset.updated_at,
        url: format!(
            "/api/realms/{}/theme/{}/assets/{}",
            realm.name, theme_id, asset.id
        ),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

fn extract_template_keys(graph: &Value) -> HashSet<String> {
    let mut keys = HashSet::new();
    let nodes = graph
        .get("nodes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    for node in nodes {
        let node_type = node
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or_default();

        let explicit = node
            .get("data")
            .and_then(|value| value.get("config"))
            .and_then(|value| {
                value
                    .get("ui")
                    .and_then(|ui| ui.get("page_key"))
                    .and_then(|page| page.as_str())
                    .or_else(|| {
                        value
                            .get("template_key")
                            .and_then(|template| template.as_str())
                    })
            });

        let template = explicit
            .map(|value| value.to_string())
            .or_else(|| default_template_key(node_type).map(|value| value.to_string()));

        if let Some(key) = template {
            keys.insert(key);
        }
    }

    keys
}

fn default_template_key(node_type: &str) -> Option<&'static str> {
    match node_type {
        "core.auth.password" => Some("login"),
        "core.auth.passkey_assert" => Some("passkey_assert"),
        "core.auth.passkey_enroll" => Some("passkey_enroll"),
        "core.auth.register" => Some("register"),
        "core.auth.forgot_credentials" => Some("forgot_credentials"),
        "core.auth.reset_password" => Some("reset_password"),
        "core.logic.recovery_issue" => Some("awaiting_action"),
        "core.logic.issue_email_otp" => Some("awaiting_action"),
        "core.oidc.consent" => Some("consent"),
        _ => None,
    }
}

pub async fn get_theme_draft_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let draft = state.theme_service.get_draft(realm.id, theme_uuid).await?;

    Ok((StatusCode::OK, Json(draft)))
}

pub async fn save_theme_draft_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
    Json(payload): Json<ThemeDraft>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    state
        .theme_service
        .save_draft(realm.id, theme_uuid, payload)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn publish_theme_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let version = state
        .theme_service
        .publish_theme(realm.id, theme_uuid)
        .await?;

    let response = ThemeVersionSummary {
        id: version.id.to_string(),
        theme_id: version.theme_id.to_string(),
        version_number: version.version_number,
        status: version.status,
        created_at: version.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn activate_theme_version_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id, version_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
    let version_uuid = Uuid::parse_str(&version_id)
        .map_err(|_| Error::Validation("Invalid version id".to_string()))?;

    state
        .theme_service
        .activate_version(realm.id, theme_uuid, version_uuid)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_theme_draft_from_version_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id, version_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
    let version_uuid = Uuid::parse_str(&version_id)
        .map_err(|_| Error::Validation("Invalid version id".to_string()))?;

    let draft = state
        .theme_service
        .start_draft_from_version(realm.id, theme_uuid, version_uuid)
        .await?;

    Ok((StatusCode::OK, Json(draft)))
}

pub async fn get_theme_version_snapshot_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id, version_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
    let version_uuid = Uuid::parse_str(&version_id)
        .map_err(|_| Error::Validation("Invalid version id".to_string()))?;

    let (version, snapshot) = state
        .theme_service
        .get_version_snapshot(realm.id, theme_uuid, version_uuid)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ThemeVersionSnapshotResponse {
            version_id: version.id.to_string(),
            theme_id: version.theme_id.to_string(),
            version_number: version.version_number,
            snapshot,
        }),
    ))
}

pub async fn list_theme_versions_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;

    let theme = state
        .theme_service
        .get_theme(realm.id, &theme_uuid)
        .await?
        .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

    let versions = state.theme_service.list_versions(&theme.id).await?;

    let response: Vec<ThemeVersionSummary> = versions
        .into_iter()
        .map(|version| ThemeVersionSummary {
            id: version.id.to_string(),
            theme_id: version.theme_id.to_string(),
            version_number: version.version_number,
            status: version.status,
            created_at: version.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_theme_asset_handler(
    State(state): State<AppState>,
    Path((realm_name, theme_id, asset_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
    let asset_uuid = Uuid::parse_str(&asset_id)
        .map_err(|_| Error::Validation("Invalid asset id".to_string()))?;

    let theme = state.theme_service.get_theme(realm.id, &theme_uuid).await?;
    if theme.is_none() {
        return Err(Error::NotFound("Theme not found".to_string()));
    }

    let asset = state
        .theme_service
        .get_asset(&theme_uuid, &asset_uuid)
        .await?
        .ok_or_else(|| Error::NotFound("Theme asset not found".to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&asset.mime_type).map_err(Error::InvalidHeader)?,
    );

    Ok((StatusCode::OK, headers, asset.data))
}
