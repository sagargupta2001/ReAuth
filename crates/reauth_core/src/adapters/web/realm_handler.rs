use crate::application::realm_service::UpdateRealmPayload;
use crate::{
    adapters::web::server::AppState, application::realm_service::CreateRealmPayload, error::Result,
};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use uuid::Uuid;

pub async fn create_realm_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRealmPayload>,
) -> Result<impl IntoResponse> {
    let realm = state.realm_service.create_realm(payload).await?;
    Ok((StatusCode::CREATED, Json(realm)))
}

pub async fn list_realms_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let realms = state.realm_service.list_realms().await?;
    Ok((StatusCode::OK, Json(realms)))
}

pub async fn update_realm_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRealmPayload>,
) -> Result<impl IntoResponse> {
    let realm = state.realm_service.update_realm(id, payload).await?;
    Ok(Json(realm))
}
