use crate::{
    adapters::web::server::AppState,
    application::realm_service::{CreateRealmPayload, RealmService},
    error::Result,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn create_realm_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRealmPayload>,
) -> Result<impl IntoResponse> {
    let realm = state.realm_service.create_realm(payload).await?;
    Ok((StatusCode::CREATED, Json(realm)))
}
