use crate::application::rbac_service::{CreateGroupPayload, CreateRolePayload};
use crate::error::Result;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn create_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRolePayload>,
) -> Result<impl IntoResponse> {
    // The `?` operator automatically converts a failure into our `Error` enum,
    // which Axum will then convert to the correct HTTP response.
    let role = state.rbac_service.create_role(payload).await?;

    // If successful, just return the `Ok` response.
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn create_group_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupPayload>,
) -> Result<impl IntoResponse> {
    let group = state.rbac_service.create_group(payload).await?;
    Ok((StatusCode::CREATED, Json(group)))
}
