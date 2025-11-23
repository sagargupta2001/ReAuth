use crate::{
    adapters::web::server::AppState, application::realm_service::CreateRealmPayload, error::Result,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

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
