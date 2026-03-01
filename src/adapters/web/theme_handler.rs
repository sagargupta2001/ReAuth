use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
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
pub struct ThemeVersionSummary {
    pub id: String,
    pub theme_id: String,
    pub version_number: i64,
    pub status: String,
    pub created_at: String,
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
        .publish_theme(realm.id, &realm.name, theme_uuid)
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
