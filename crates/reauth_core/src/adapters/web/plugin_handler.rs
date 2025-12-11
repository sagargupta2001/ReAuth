use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use manager::grpc::plugin::v1::greeter_client::GreeterClient;
use manager::grpc::plugin::v1::HelloRequest;

/// Handler to provide the list of active plugin manifests to the UI.
pub async fn get_plugin_manifests(State(state): State<AppState>) -> impl IntoResponse {
    // Call the new public API on the plugin manager
    match state.plugin_manager.get_plugin_statuses().await {
        Ok(plugin_statuses) => {
            // Return the full list of plugins (both active and inactive)
            Json(plugin_statuses).into_response()
        }
        Err(e) => {
            // Handle any errors during the scan
            tracing::error!("Failed to get plugin statuses: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve plugin list".to_string(),
            )
                .into_response()
        }
    }
}

/// Handler that proxies a generic plugin API call to the correct plugin's gRPC backend.
pub async fn plugin_proxy_handler(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> impl IntoResponse {
    // Use the new public method instead of accessing the field directly.
    if let Some(channel) = state
        .plugin_manager
        .get_active_plugin_channel(&plugin_id)
        .await
    {
        let mut client = GreeterClient::new(channel); // The channel is already cloned
        let request = tonic::Request::new(HelloRequest {
            name: "Proxied User".to_string(),
        });
        match client.say_hello(request).await {
            Ok(response) => Json(response.into_inner()).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        // The plugin is either not installed or not running.
        // 503 Service Unavailable is more accurate than 404 Not Found.
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Plugin is not active".to_string(),
        )
            .into_response()
    }
}

pub async fn enable_plugin_handler(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> impl IntoResponse {
    match state.plugin_manager.enable_plugin(&plugin_id).await {
        Ok(()) => (StatusCode::OK, Json("Plugin enabled")).into_response(),
        Err(e) => {
            tracing::error!("Failed to enable plugin {}: {}", plugin_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn disable_plugin_handler(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> impl IntoResponse {
    match state.plugin_manager.disable_plugin(&plugin_id).await {
        Ok(()) => (StatusCode::OK, Json("Plugin disabled")).into_response(),
        Err(e) => {
            tracing::error!("Failed to disable plugin {}: {}", plugin_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
