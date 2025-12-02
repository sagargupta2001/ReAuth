use crate::{
    adapters::web::server::AppState,
    domain::pagination::PageRequest,
    error::{Error, Result},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use uuid::Uuid;

pub async fn list_sessions_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.auth_service.list_sessions(realm.id, req).await?;

    Ok((StatusCode::OK, Json(response)))
}

pub async fn revoke_session_handler(
    State(state): State<AppState>,
    Path((_realm, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    state.auth_service.logout(id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}
