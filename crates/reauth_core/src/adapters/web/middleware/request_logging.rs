use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;
use tracing::info;

pub async fn log_api_request(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_string());

    let response = next.run(req).await;

    let status = response.status();
    let duration_ms = start.elapsed().as_millis() as u64;
    let route = matched_path.as_deref().unwrap_or(&path);

    info!(
        message = "api.request",
        method = %method,
        route = %route,
        path = %path,
        status = status.as_u16(),
        duration_ms = duration_ms,
    );

    response
}
