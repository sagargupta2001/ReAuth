use axum::{
    extract::State,
    http::{Request, StatusCode, Uri},
    response::IntoResponse,
};
use manager::PluginManager;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

// Import the application services and handlers
use crate::adapters::web::router::create_router;
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
pub(crate) mod ui_handler {
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
pub(crate) mod ui_handler {
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

    let app = create_router(app_state, plugins_path);

    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>()?,
        settings.server.port,
    ));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
