use axum::{
    extract::{Path, State},
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Json},
    routing::{get, get_service, post},
    Router,
};
use manager::{
    grpc::plugin::v1::{greeter_client::GreeterClient, HelloRequest},
    Manifest, PluginManager,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

// Import the application services and handlers
use crate::application::{rbac_service::RbacService, user_service::UserService};
use crate::config::Settings;
use crate::adapters::web::{rbac_handler, user_handler};

/// AppState is the single, shared state for the entire Axum application.
/// It holds all necessary services and configurations.
#[derive(Clone)]
pub(crate) struct AppState {
    plugin_manager: PluginManager,
    settings: Settings,
    pub(crate) user_service: Arc<UserService>,
    pub(crate) rbac_service: Arc<RbacService>,
}

#[cfg(not(feature = "embed-ui"))]
mod ui_handler {
    use super::*;
    use axum::body::Body;
    /// Proxies all UI requests to the React dev server (e.g., http://localhost:5173)
    pub async fn static_handler(
        State(state): State<AppState>, // Handlers now use the new AppState
        uri: Uri,
        _req: Request<Body>,
    ) -> impl IntoResponse {
        let url = format!("{}{}", state.settings.ui.dev_url, uri.path_and_query().map(|u| u.as_str()).unwrap_or("/"));
        match reqwest::get(&url).await {
            Ok(resp) => (resp.status(), resp.headers().clone(), resp.bytes().await.unwrap_or_default()).into_response(),
            Err(_) => (StatusCode::BAD_GATEWAY, "React dev server not running").into_response(),
        }
    }
}

#[cfg(feature = "embed-ui")]
mod ui_handler {
    use super::*;
    use rust_embed::Embed;
    use axum::http::{header, HeaderValue, StatusCode};

    #[derive(Embed)]
    #[folder = "../../ui/dist/"] // Corrected path relative to crate root
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

// --- API Handlers ---

/// Handler to provide the list of active plugin manifests to the UI.
async fn get_plugin_manifests(State(state): State<AppState>) -> impl IntoResponse {
    let instances = state.plugin_manager.instances.lock().await;
    let manifests: Vec<Manifest> = instances.values().map(|inst| inst.manifest.clone()).collect();
    Json(manifests)
}

/// Handler that proxies a generic plugin API call to the correct plugin's gRPC backend.
async fn plugin_proxy_handler(
    State(state): State<AppState>,
    Path(plugin_id): Path<String>,
) -> impl IntoResponse {
    let instances = state.plugin_manager.instances.lock().await;
    if let Some(instance) = instances.get(&plugin_id) {
        let mut client = GreeterClient::new(instance.grpc_channel.clone());
        let request = tonic::Request::new(HelloRequest { name: "Proxied User".to_string() });
        match client.say_hello(request).await {
            Ok(response) => Json(response.into_inner()).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Plugin not found".to_string()).into_response()
    }
}

/// The main server startup function.
/// This now accepts all the application's dependencies from `main.rs`.
pub async fn start_server(
    settings: Settings,
    plugin_manager: PluginManager,
    plugins_path: PathBuf,
    user_service: Arc<UserService>,
    rbac_service: Arc<RbacService>,
) -> anyhow::Result<()> {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // Create the single, unified AppState
    let app_state = AppState {
        plugin_manager,
        settings: settings.clone(),
        user_service,
        rbac_service,
    };

    // --- API Router Definition ---
    // We create sub-routers for each part of the API for cleanliness.

    let user_api = Router::new()
        .route("/", post(user_handler::create_user_handler));

    let rbac_api = Router::new()
        .route("/roles", post(rbac_handler::create_role_handler))
        .route("/groups", post(rbac_handler::create_group_handler));

    let plugin_api = Router::new()
        .route("/manifests", get(get_plugin_manifests))
        .route("/{id}/say-hello", get(plugin_proxy_handler));

    // Combine all API routers under the /api prefix
    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/users", user_api)
        .nest("/rbac", rbac_api)
        .nest("/plugins", plugin_api);

    // --- Main Application Router ---
    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/plugins", get_service(ServeDir::new(plugins_path)))
        .fallback(ui_handler::static_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from((settings.server.host.parse::<std::net::IpAddr>()?, settings.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}