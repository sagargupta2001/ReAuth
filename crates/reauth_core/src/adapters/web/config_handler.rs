use crate::bootstrap::initialize::apply_settings_update;
use crate::config::Settings;
use crate::error::Result;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct ReloadConfigResponse {
    status: &'static str,
    message: String,
}

pub async fn reload_config_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let Some(config_path) = Settings::resolve_config_watch_path() else {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ReloadConfigResponse {
                status: "error",
                message: "No config file was found to reload.".to_string(),
            }),
        ));
    };

    if !config_path.exists() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ReloadConfigResponse {
                status: "error",
                message: format!("Config file does not exist: {}", config_path.display()),
            }),
        ));
    }

    let new_settings = Settings::new()?;
    apply_settings_update(&state.settings, new_settings).await;

    Ok((
        StatusCode::OK,
        Json(ReloadConfigResponse {
            status: "ok",
            message: "Config reloaded.".to_string(),
        }),
    ))
}
