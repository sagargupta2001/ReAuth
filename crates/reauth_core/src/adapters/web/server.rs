use axum::{
    extract::State,
    http::{Request, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};
use manager::PluginManager;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

// Import the application services and handlers
use crate::adapters::web::{
    auth_handler, log_stream_handler, plugin_handler, rbac_handler, realm_handler, user_handler,
};
use crate::application::auth_service::AuthService;
use crate::application::{
    rbac_service::RbacService, realm_service::RealmService, user_service::UserService,
};
use crate::config::Settings;
use manager::log_bus::LogSubscriber;

/// AppState is the single, shared state for the entire Axum application.
/// It holds all necessary services and configurations.
#[derive(Clone)]
pub struct AppState {
    pub(crate) plugin_manager: PluginManager,
    settings: Settings,
    pub(crate) user_service: Arc<UserService>,
    pub(crate) rbac_service: Arc<RbacService>,
    pub auth_service: Arc<AuthService>,
    pub realm_service: Arc<RealmService>,
    pub log_subscriber: Arc<dyn LogSubscriber>,
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
        let url = format!(
            "{}{}",
            state.settings.ui.dev_url,
            uri.path_and_query().map(|u| u.as_str()).unwrap_or("/")
        );
        match reqwest::get(&url).await {
            Ok(resp) => (
                resp.status(),
                resp.headers().clone(),
                resp.bytes().await.unwrap_or_default(),
            )
                .into_response(),
            Err(_) => (StatusCode::BAD_GATEWAY, "React dev server not running").into_response(),
        }
    }
}

#[cfg(feature = "embed-ui")]
mod ui_handler {
    use super::*;
    use axum::http::{header, HeaderValue, StatusCode};
    use rust_embed::Embed;

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
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(mime_type.essence_str()).unwrap(),
                    )],
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

/// The main server startup function.
/// This now accepts all the application's dependencies from `main.rs`.
pub async fn start_server(
    settings: Settings,
    plugin_manager: PluginManager,
    plugins_path: PathBuf,
    user_service: Arc<UserService>,
    rbac_service: Arc<RbacService>,
    auth_service: Arc<AuthService>,
    realm_service: Arc<RealmService>,
    log_subscriber: Arc<dyn LogSubscriber>,
) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create the single, unified AppState
    let app_state = AppState {
        plugin_manager,
        settings: settings.clone(),
        user_service,
        rbac_service,
        auth_service,
        realm_service,
        log_subscriber,
    };

    // --- API Router Definition ---
    // We create sub-routers for each part of the API for cleanliness.

    let user_api = Router::new().route("/", post(user_handler::create_user_handler));

    let rbac_api = Router::new()
        .route("/roles", post(rbac_handler::create_role_handler))
        .route("/groups", post(rbac_handler::create_group_handler));

    let plugin_api = Router::new()
        .route("/manifests", get(plugin_handler::get_plugin_manifests))
        .route("/{id}/say-hello", get(plugin_handler::plugin_proxy_handler))
        .route("/{id}/enable", post(plugin_handler::enable_plugin_handler))
        .route(
            "/{id}/disable",
            post(plugin_handler::disable_plugin_handler),
        );

    let auth_api = Router::new().route("/login", post(auth_handler::login_handler));

    let realm_api = Router::new().route("/", post(realm_handler::create_realm_handler));

    // Combine all API routers under the /api prefix
    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/logs/ws", get(log_stream_handler::log_stream_handler))
        .nest("/users", user_api)
        .nest("/rbac", rbac_api)
        .nest("/realms", realm_api)
        .nest("/plugins", plugin_api)
        .nest("/auth", auth_api);

    // --- Main Application Router ---
    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/plugins", get_service(ServeDir::new(plugins_path)))
        .fallback(ui_handler::static_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>()?,
        settings.server.port,
    ));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
