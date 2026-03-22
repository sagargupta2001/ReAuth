use crate::application::realm_recovery_settings_service::UpdateRealmRecoverySettingsPayload;
use crate::domain::realm_recovery_settings::RealmRecoverySettings;
use crate::{error::Result, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RealmRecoverySettingsResponse {
    pub realm_id: Uuid,
    pub token_ttl_minutes: i64,
    pub rate_limit_max: i64,
    pub rate_limit_window_minutes: i64,
    pub revoke_sessions_on_reset: bool,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
}

impl From<RealmRecoverySettings> for RealmRecoverySettingsResponse {
    fn from(settings: RealmRecoverySettings) -> Self {
        Self {
            realm_id: settings.realm_id,
            token_ttl_minutes: settings.token_ttl_minutes,
            rate_limit_max: settings.rate_limit_max,
            rate_limit_window_minutes: settings.rate_limit_window_minutes,
            revoke_sessions_on_reset: settings.revoke_sessions_on_reset,
            email_subject: settings.email_subject,
            email_body: settings.email_body,
        }
    }
}

pub async fn get_realm_recovery_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_recovery_settings_service
        .get_settings(id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RealmRecoverySettingsResponse::from(settings)),
    ))
}

pub async fn update_realm_recovery_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmRecoverySettingsPayload>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_recovery_settings_service
        .update_settings(id, payload)
        .await?;
    Ok(Json(RealmRecoverySettingsResponse::from(settings)))
}
