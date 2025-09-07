use axum::{
    body::Body,
    http::{Request, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;

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

pub async fn start_server(db: Database) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .fallback(ui_handler::static_handler)
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
