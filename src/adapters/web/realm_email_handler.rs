use crate::adapters::web::validation::ValidatedJson;
use crate::application::realm_email_settings_service::{
    apply_payload, validate_settings, UpdateRealmEmailSettingsPayload,
};
use crate::domain::realm_email_settings::RealmEmailSettings;
use crate::{error::Result, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize)]
pub struct RealmEmailSettingsResponse {
    pub realm_id: Uuid,
    pub enabled: bool,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_username: Option<String>,
    pub smtp_security: String,
    pub smtp_password_set: bool,
}

impl From<RealmEmailSettings> for RealmEmailSettingsResponse {
    fn from(settings: RealmEmailSettings) -> Self {
        let smtp_password_set = settings
            .smtp_password
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        Self {
            realm_id: settings.realm_id,
            enabled: settings.enabled,
            from_address: settings.from_address,
            from_name: settings.from_name,
            reply_to_address: settings.reply_to_address,
            smtp_host: settings.smtp_host,
            smtp_port: settings.smtp_port,
            smtp_username: settings.smtp_username,
            smtp_security: settings.smtp_security,
            smtp_password_set,
        }
    }
}

pub async fn get_realm_email_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let settings = state.realm_email_settings_service.get_settings(id).await?;
    Ok((
        StatusCode::OK,
        Json(RealmEmailSettingsResponse::from(settings)),
    ))
}

pub async fn update_realm_email_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmEmailSettingsPayload>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_email_settings_service
        .update_settings(id, payload)
        .await?;
    Ok(Json(RealmEmailSettingsResponse::from(settings)))
}

#[derive(Deserialize, Validate)]
pub struct TestEmailPayload {
    #[validate(email(message = "Recipient email is invalid"))]
    pub to_address: String,
    #[serde(flatten)]
    pub settings: UpdateRealmEmailSettingsPayload,
}

pub async fn test_realm_email_settings_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<TestEmailPayload>,
) -> Result<impl IntoResponse> {
    let mut settings = state.realm_email_settings_service.get_settings(id).await?;
    apply_payload(&mut settings, payload.settings);
    settings.enabled = true;
    validate_settings(&settings)?;

    state
        .email_delivery_service
        .send_test_email(&id, settings, &payload.to_address)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
