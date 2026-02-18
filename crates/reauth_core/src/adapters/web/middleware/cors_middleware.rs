use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;
use url::Url;

pub async fn dynamic_cors_guard(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let settings = state.settings.read().await.clone();
    let origin_header = match req.headers().get("origin") {
        Some(h) => h,
        None => return next.run(req).await, // Non-browser request (e.g. Curl), let it pass
    };

    let origin_str = match origin_header.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap()
        }
    };

    // 1. Allow configured UI + server origins from settings
    if is_allowed_origin(&settings, &origin_str) {
        return allow_response(next.run(req).await, &origin_str);
    }

    // 2. Check DB for this origin
    let exists = state
        .oidc_service
        .is_origin_allowed(&origin_str)
        .await
        .unwrap_or(false);

    if exists {
        return allow_response(next.run(req).await, &origin_str);
    }

    warn!("Blocked CORS request from: {}", origin_str);
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Body::from("CORS Origin Not Allowed"))
        .unwrap()
}

fn allow_response(mut response: Response, origin: &str) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_str(origin).unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Credentials",
        HeaderValue::from_static("true"),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Authorization, Content-Type, Cookie, Accept"),
    );
    response
}

fn is_allowed_origin(settings: &crate::config::Settings, origin: &str) -> bool {
    let mut allowed = Vec::new();

    for bind_origin in bind_origins(settings) {
        allowed.push(bind_origin);
    }

    if let Some(server_origin) = normalize_origin(&settings.server.public_url) {
        allowed.push(server_origin);
    }

    if let Some(ui_origin) = normalize_origin(&settings.ui.dev_url) {
        allowed.push(ui_origin);
    }

    for configured_origin in &settings.cors.allowed_origins {
        if let Some(origin) = normalize_origin(configured_origin) {
            allowed.push(origin);
        }
    }

    for configured_origin in &settings.default_oidc_client.web_origins {
        if let Some(origin) = normalize_origin(configured_origin) {
            allowed.push(origin);
        }
    }

    allowed.iter().any(|allowed_origin| allowed_origin == origin)
}

fn normalize_origin(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(parsed) = Url::parse(trimmed) {
        return Some(parsed.origin().unicode_serialization());
    }

    None
}

fn bind_origins(settings: &crate::config::Settings) -> Vec<String> {
    let mut origins = Vec::new();
    let scheme = settings.server.scheme.trim();
    let host = settings.server.host.trim();

    if scheme.is_empty() || host.is_empty() {
        return origins;
    }

    let port = settings.server.port;
    origins.push(format!("{}://{}:{}", scheme, host, port));

    if host == "127.0.0.1" {
        origins.push(format!("{}://localhost:{}", scheme, port));
    }

    origins
}
