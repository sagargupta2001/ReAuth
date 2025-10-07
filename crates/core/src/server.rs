use axum::{body::Body, http::{Request, Uri}, response::IntoResponse, routing::get, Json, Router};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;
use axum::extract::State;
use axum::routing::get_service;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use reauth_plugin_manager::{Manifest, PluginManager};
use crate::database::Database;

#[cfg(feature = "embed-ui")]
mod ui_handler {
    use super::*;
    use rust_embed::Embed;
    use axum::http::{header, HeaderValue, StatusCode};

    #[derive(Embed)]
    #[folder = "../ui/dist/"]
    pub struct UiAssets;

    pub async fn static_handler(uri: Uri) -> impl IntoResponse {
        let mut path = uri.path().trim_start_matches('/').to_string();
        if path.is_empty() {
            path = "index.html".to_string();
        }

        match UiAssets::get(&path) {
            Some(content) => {
                let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, HeaderValue::from_str(mime_type.essence_str()).unwrap())],
                    content.data.into_owned(),
                )
            }
            None => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))],
                UiAssets::get("index.html").unwrap().data.into_owned(),
            ),
        }
    }
}

#[cfg(not(feature = "embed-ui"))]
mod ui_handler {
    use super::*;
    use axum::http::StatusCode;

    /// Proxy all UI requests to React dev server (http://localhost:5173)
    pub async fn static_handler(uri: Uri, _: Request<Body>) -> impl IntoResponse {
        let url = format!("http://localhost:5173{}", uri.path_and_query().map(|u| u.as_str()).unwrap_or("/"));

        match reqwest::get(&url).await {
            Ok(resp) => {
                let status = resp.status();
                let headers = resp.headers().clone();
                let body = resp.bytes().await.unwrap_or_default();
                (status, headers, body).into_response()
            }
            Err(_) => (
                StatusCode::BAD_GATEWAY,
                "React dev server not running".to_string(),
            )
                .into_response(),
        }
    }
}

async fn get_plugin_manifests(
    State(plugin_manager): State<PluginManager>,
) -> impl IntoResponse {
    let instances = plugin_manager.instances.lock().await;
    let manifests: Vec<Manifest> = instances.values().map(|inst| inst.manifest.clone()).collect();
    Json(manifests)
}

pub async fn start_server(db: Database, plugin_manager: PluginManager) -> anyhow::Result<()> {

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Route to serve static assets from plugin directories
    let plugins_asset_route =
        Router::new().nest_service("/plugins", get_service(ServeDir::new("plugins")));

    // --- REFACTORED ROUTING LOGIC ---
    // 1. Create a router for all API endpoints.
    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/plugins/manifests", get(get_plugin_manifests));

    // 2. Create the main application router.
    let app = Router::new()
        .nest("/api", api_router) // Nest all API routes under the `/api` prefix.
        .merge(plugins_asset_route)
        .fallback(ui_handler::static_handler) // This fallback now correctly ignores `/api` routes.
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(plugin_manager)
        .with_state(db);
    // --- END OF REFACTOR ---

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
