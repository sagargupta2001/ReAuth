use crate::application::realm_security_headers_service::UpdateRealmSecurityHeadersPayload;
use crate::domain::realm_security_headers::RealmSecurityHeaders;
use crate::{error::Result, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct RealmSecurityHeadersResponse {
    pub realm_id: Uuid,
    pub x_frame_options: Option<String>,
    pub content_security_policy: Option<String>,
    pub x_content_type_options: Option<String>,
    pub referrer_policy: Option<String>,
    pub strict_transport_security: Option<String>,
}

impl From<RealmSecurityHeaders> for RealmSecurityHeadersResponse {
    fn from(settings: RealmSecurityHeaders) -> Self {
        Self {
            realm_id: settings.realm_id,
            x_frame_options: settings.x_frame_options,
            content_security_policy: settings.content_security_policy,
            x_content_type_options: settings.x_content_type_options,
            referrer_policy: settings.referrer_policy,
            strict_transport_security: settings.strict_transport_security,
        }
    }
}

pub async fn get_realm_security_headers_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_security_headers_service
        .get_settings(id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RealmSecurityHeadersResponse::from(settings)),
    ))
}

pub async fn update_realm_security_headers_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmSecurityHeadersPayload>,
) -> Result<impl IntoResponse> {
    let settings = state
        .realm_security_headers_service
        .update_settings(id, payload)
        .await?;
    Ok(Json(RealmSecurityHeadersResponse::from(settings)))
}
