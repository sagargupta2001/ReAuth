use crate::application::realm_idp_settings_service::UpdateRealmIdpSettingsPayload;
use crate::domain::realm_idp_settings::RealmIdpSettings;
use crate::{error::Result, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RealmIdpSettingsResponse {
    pub realm_id: Uuid,
    pub oauth_start_rate_limit_max: i64,
    pub oauth_start_rate_limit_window_minutes: i64,
}

impl From<RealmIdpSettings> for RealmIdpSettingsResponse {
    fn from(settings: RealmIdpSettings) -> Self {
        Self {
            realm_id: settings.realm_id,
            oauth_start_rate_limit_max: settings.oauth_start_rate_limit_max,
            oauth_start_rate_limit_window_minutes: settings.oauth_start_rate_limit_window_minutes,
        }
    }
}

pub async fn get_realm_idp_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let settings = state.realm_idp_settings_service.get_settings(id).await?;
    Ok((
        StatusCode::OK,
        Json(RealmIdpSettingsResponse::from(settings)),
    ))
}

pub async fn update_realm_idp_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmIdpSettingsPayload>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_idp_settings_service
        .update_settings(id, payload)
        .await?;
    Ok(Json(RealmIdpSettingsResponse::from(settings)))
}
