use crate::adapters::web::router::create_router;

use crate::AppState;
use axum::http::Uri;
use axum::response::IntoResponse;
use std::net::SocketAddr;

#[cfg(not(feature = "embed-ui"))]
pub(crate) mod ui_handler {
    use super::*;
    use crate::AppState;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};

    /// Proxies all UI requests to the React dev server (e.g., http://localhost:5173)
    pub async fn static_handler(
        State(state): State<AppState>, // Handlers now use the new AppState
        uri: Uri,
        _req: Request<Body>,
    ) -> impl IntoResponse {
        let dev_url = {
            let settings = state.settings.read().await;
            settings.ui.dev_url.clone()
        };
        let url = format!(
            "{}{}",
            dev_url,
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
pub async fn start_server(app_state: AppState) -> anyhow::Result<()> {
    // Extract settings for binding
    let settings = app_state.settings.read().await.clone();

    // Extract plugins path (Assuming it was added to AppState in bootstrap,
    // otherwise pass it as a 2nd argument)
    let plugins_path = app_state.plugins_path.clone();

    // Create Router
    let app = create_router(app_state, plugins_path);

    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>()?,
        settings.server.port,
    ));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
