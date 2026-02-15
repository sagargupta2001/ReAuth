use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

pub async fn dynamic_cors_guard(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
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

    // 1. Allow own origin (Dashboard)
    // You might want to make this configurable via env var
    if origin_str == "http://localhost:3000" {
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
