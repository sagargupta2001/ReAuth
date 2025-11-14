use crate::{
    adapters::web::server::AppState,
    application::auth_service::LoginPayload,
    error::Result, // Use the app's Result
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    // Call the application service
    let session_token: String = state.auth_service.login(payload).await?;

    // On success, return the token
    Ok((StatusCode::OK, Json(session_token)))
}
