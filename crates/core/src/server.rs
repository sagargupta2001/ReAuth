use crate::config::Settings;
use crate::database::Database;
use axum::{
    extract::{Path, State},
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Json},
    routing::{get, get_service},
    Router,
};
use manager::{
    grpc::plugin::v1::{greeter_client::GreeterClient, HelloRequest},
    Manifest, PluginManager,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

#[derive(Clone)]
struct AppState {
    db: Database,
    plugin_manager: PluginManager,
    settings: Settings,
}

#[cfg(not(feature = "embed-ui"))]
mod ui_handler {
    use super::*;
    use axum::body::Body;
    pub async fn static_handler(
        State(state): State<AppState>,
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
    #[folder = "../../ui/dist/"]
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

async fn get_plugin_manifests(State(state): State<AppState>) -> impl IntoResponse {
    let instances = state.plugin_manager.instances.lock().await;
    let manifests: Vec<Manifest> = instances.values().map(|inst| inst.manifest.clone()).collect();
    Json(manifests)
}

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

pub async fn start_server(
    db: Database,
    plugin_manager: PluginManager,
    settings: Settings,
    plugins_path: PathBuf,
) -> anyhow::Result<()> {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // Create the single, unified AppState
    let app_state = AppState { db, plugin_manager, settings: settings.clone() };

    let plugins_asset_route = Router::new().nest_service("/plugins", get_service(ServeDir::new(plugins_path)));

    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/plugins/manifests", get(get_plugin_manifests))
        .route("/plugins/{id}/say-hello", get(plugin_proxy_handler));

    let app = Router::new()
        .nest("/api", api_router)
        .merge(plugins_asset_route)
        .fallback(ui_handler::static_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state); // Pass the unified state to the router

    let addr = SocketAddr::from((settings.server.host.parse::<std::net::IpAddr>()?, settings.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}