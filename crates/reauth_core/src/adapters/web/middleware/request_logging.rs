use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";
const CORRELATION_ID_HEADER: &str = "x-correlation-id";

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn log_api_request(mut req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let had_correlation_id = req.headers().contains_key(CORRELATION_ID_HEADER);
    let had_request_id = req.headers().contains_key(REQUEST_ID_HEADER);
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_string());
    let request_id = extract_or_generate_request_id(&req);
    let request_id_header = HeaderValue::from_str(&request_id).ok();

    req.extensions_mut().insert(RequestId(request_id.clone()));
    if let Some(header_value) = request_id_header.clone() {
        req.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }

    let response = next.run(req).await;

    let mut response = response;
    let status = response.status();
    let duration_ms = start.elapsed().as_millis() as u64;
    let route = matched_path.as_deref().unwrap_or(&path);

    if let Some(header_value) = request_id_header.clone() {
        response
            .headers_mut()
            .insert(REQUEST_ID_HEADER, header_value);
    }
    if had_correlation_id && !had_request_id {
        if let Some(header_value) = request_id_header {
            response
                .headers_mut()
                .insert(CORRELATION_ID_HEADER, header_value);
        }
    }

    info!(
        request_id = %request_id,
        method = %method,
        route = %route,
        path = %path,
        status = status.as_u16(),
        duration_ms = duration_ms,
        "api.request"
    );

    response
}

fn extract_or_generate_request_id(req: &Request<Body>) -> String {
    if let Some(value) =
        header_value(req, REQUEST_ID_HEADER).or_else(|| header_value(req, CORRELATION_ID_HEADER))
    {
        return value;
    }

    Uuid::new_v4().to_string()
}

fn header_value(req: &Request<Body>, name: &str) -> Option<String> {
    let header = req.headers().get(name)?;
    let value = header.to_str().ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return None;
    }
    Some(trimmed.to_string())
}
