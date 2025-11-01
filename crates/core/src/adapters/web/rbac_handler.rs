use crate::application::rbac_service::{CreateGroupPayload, CreateRolePayload, RbacService};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use crate::adapters::web::server::AppState;

pub async fn create_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateRolePayload>,
) -> impl IntoResponse {
    match state.rbac_service.create_role(payload).await {
        Ok(role) => (StatusCode::CREATED, Json(role)).into_response(),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn create_group_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupPayload>,
) -> impl IntoResponse {
    match state.rbac_service.create_group(payload).await {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}