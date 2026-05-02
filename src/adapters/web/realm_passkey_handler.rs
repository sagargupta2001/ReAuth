use crate::application::flow_manager::templates::FlowTemplates;
use crate::application::flow_manager::UpdateDraftRequest;
use crate::application::passkey_analytics_service::PasskeyAnalyticsSnapshot;
use crate::application::realm_passkey_settings_service::UpdateRealmPasskeySettingsPayload;
use crate::domain::flow::models::FlowDraft;
use crate::domain::realm_passkey_settings::RealmPasskeySettings;
use crate::{error::Error, error::Result, AppState};
use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RealmPasskeySettingsResponse {
    pub realm_id: Uuid,
    pub enabled: bool,
    pub allow_password_fallback: bool,
    pub discoverable_preferred: bool,
    pub challenge_ttl_secs: i64,
    pub reauth_max_age_secs: i64,
}

impl From<RealmPasskeySettings> for RealmPasskeySettingsResponse {
    fn from(settings: RealmPasskeySettings) -> Self {
        Self {
            realm_id: settings.realm_id,
            enabled: settings.enabled,
            allow_password_fallback: settings.allow_password_fallback,
            discoverable_preferred: settings.discoverable_preferred,
            challenge_ttl_secs: settings.challenge_ttl_secs,
            reauth_max_age_secs: settings.reauth_max_age_secs,
        }
    }
}

#[derive(Deserialize)]
pub struct ApplyRecommendedPasskeyFlowPayload {
    pub enable_passkeys: Option<bool>,
}

#[derive(Serialize)]
pub struct ApplyRecommendedPasskeyFlowResponse {
    pub settings: RealmPasskeySettingsResponse,
    pub browser_flow_version_id: String,
    pub browser_flow_version_number: i32,
}

#[derive(Serialize)]
pub struct ApplyRecommendedPasskeyRegistrationFlowResponse {
    pub settings: RealmPasskeySettingsResponse,
    pub registration_flow_version_id: String,
    pub registration_flow_version_number: i32,
}

#[derive(Debug, Deserialize)]
pub struct PasskeyAnalyticsQuery {
    pub window_hours: Option<i64>,
    pub recent_limit: Option<usize>,
}

pub async fn get_realm_passkey_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_passkey_settings_service
        .get_settings(id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RealmPasskeySettingsResponse::from(settings)),
    ))
}

pub async fn update_realm_passkey_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmPasskeySettingsPayload>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_passkey_settings_service
        .update_settings(id, payload)
        .await?;
    Ok(Json(RealmPasskeySettingsResponse::from(settings)))
}

pub async fn get_realm_passkey_analytics_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PasskeyAnalyticsQuery>,
) -> Result<impl IntoResponse> {
    let window_hours = query.window_hours.unwrap_or(24);
    let recent_limit = query.recent_limit.unwrap_or(10);
    let snapshot: PasskeyAnalyticsSnapshot = state
        .passkey_analytics_service
        .snapshot(id, window_hours, recent_limit)
        .await?;
    Ok((StatusCode::OK, Json(snapshot)))
}

pub async fn apply_recommended_passkey_browser_flow_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApplyRecommendedPasskeyFlowPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_id(id)
        .await?
        .ok_or(Error::RealmNotFound(id.to_string()))?;
    let browser_flow_id = realm
        .browser_flow_id
        .as_deref()
        .ok_or_else(|| Error::Validation("Realm has no browser flow configured".to_string()))
        .and_then(|value| {
            Uuid::parse_str(value)
                .map_err(|_| Error::System("Realm browser flow id is invalid".to_string()))
        })?;

    let should_enable = payload.enable_passkeys.unwrap_or(true);
    let settings = if should_enable {
        state
            .realm_passkey_settings_service
            .update_settings(
                id,
                UpdateRealmPasskeySettingsPayload {
                    enabled: Some(true),
                    allow_password_fallback: Some(true),
                    discoverable_preferred: None,
                    challenge_ttl_secs: None,
                    reauth_max_age_secs: None,
                },
            )
            .await?
    } else {
        state
            .realm_passkey_settings_service
            .get_settings(id)
            .await?
    };

    let graph = FlowTemplates::passkey_first_browser_flow();
    state
        .flow_manager
        .update_draft(
            browser_flow_id,
            UpdateDraftRequest {
                name: None,
                description: Some("Recommended passkey-first browser flow".to_string()),
                graph_json: Some(graph.clone()),
            },
        )
        .await?;

    let version = state
        .flow_manager
        .publish_flow(realm.id, browser_flow_id)
        .await?;
    let browser_flow_name = state
        .flow_service
        .list_flows(realm.id)
        .await?
        .into_iter()
        .find(|flow| flow.id == browser_flow_id)
        .map(|flow| flow.name)
        .unwrap_or_else(|| "browser-login".to_string());

    // Keep draft aligned with the active version so the editor immediately reflects the preset.
    state
        .flow_manager
        .create_draft_with_id(FlowDraft {
            id: browser_flow_id,
            realm_id: realm.id,
            name: browser_flow_name,
            description: Some("Recommended passkey-first browser flow".to_string()),
            graph_json: graph.to_string(),
            flow_type: "browser".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApplyRecommendedPasskeyFlowResponse {
            settings: RealmPasskeySettingsResponse::from(settings),
            browser_flow_version_id: version.id,
            browser_flow_version_number: version.version_number,
        }),
    ))
}

pub async fn apply_recommended_passkey_registration_flow_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_id(id)
        .await?
        .ok_or(Error::RealmNotFound(id.to_string()))?;
    let registration_flow_id = realm
        .registration_flow_id
        .as_deref()
        .ok_or_else(|| Error::Validation("Realm has no registration flow configured".to_string()))
        .and_then(|value| {
            Uuid::parse_str(value)
                .map_err(|_| Error::System("Realm registration flow id is invalid".to_string()))
        })?;

    let settings = state
        .realm_passkey_settings_service
        .update_settings(
            id,
            UpdateRealmPasskeySettingsPayload {
                enabled: Some(true),
                allow_password_fallback: Some(true),
                discoverable_preferred: None,
                challenge_ttl_secs: None,
                reauth_max_age_secs: None,
            },
        )
        .await?;

    let graph = FlowTemplates::passkey_enroll_registration_flow();
    state
        .flow_manager
        .update_draft(
            registration_flow_id,
            UpdateDraftRequest {
                name: None,
                description: Some(
                    "Recommended registration flow with passkey enrollment".to_string(),
                ),
                graph_json: Some(graph.clone()),
            },
        )
        .await?;
    let version = state
        .flow_manager
        .publish_flow(realm.id, registration_flow_id)
        .await?;
    let registration_flow_name = state
        .flow_service
        .list_flows(realm.id)
        .await?
        .into_iter()
        .find(|flow| flow.id == registration_flow_id)
        .map(|flow| flow.name)
        .unwrap_or_else(|| "registration".to_string());

    state
        .flow_manager
        .create_draft_with_id(FlowDraft {
            id: registration_flow_id,
            realm_id: realm.id,
            name: registration_flow_name,
            description: Some("Recommended registration flow with passkey enrollment".to_string()),
            graph_json: graph.to_string(),
            flow_type: "registration".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(ApplyRecommendedPasskeyRegistrationFlowResponse {
            settings: RealmPasskeySettingsResponse::from(settings),
            registration_flow_version_id: version.id,
            registration_flow_version_number: version.version_number,
        }),
    ))
}
