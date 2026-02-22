use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::{header, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use rand::RngCore;
use serde_json::Value;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";
const CORRELATION_ID_HEADER: &str = "x-correlation-id";
const TRACEPARENT_HEADER: &str = "traceparent";
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

#[derive(Clone, Debug)]
pub struct TraceParent {
    pub version: String,
    pub trace_id: String,
    pub parent_id: String,
    pub flags: String,
}

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
    let traceparent = extract_or_generate_traceparent(&req);
    let traceparent_value = HeaderValue::from_str(&traceparent.as_header_value()).ok();

    req.extensions_mut().insert(RequestId(request_id.clone()));
    req.extensions_mut().insert(traceparent.clone());
    if let Some(header_value) = request_id_header.clone() {
        req.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }
    if let Some(header_value) = traceparent_value.clone() {
        req.headers_mut().insert(TRACEPARENT_HEADER, header_value);
    }

    let response = next.run(req).await;
    let mut response = response;
    let status = response.status();
    let duration_ms = start.elapsed().as_millis() as u64;
    let route = matched_path.as_deref().unwrap_or(&path);

    if status.is_client_error() || status.is_server_error() {
        response = inject_request_id_into_error(response, &request_id).await;
    }

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
    if let Some(header_value) = traceparent_value {
        response
            .headers_mut()
            .insert(TRACEPARENT_HEADER, header_value);
    }

    info!(
        request_id = %request_id,
        trace_id = %traceparent.trace_id,
        span_id = %traceparent.parent_id,
        method = %method,
        route = %route,
        path = %path,
        status = status.as_u16(),
        duration_ms = duration_ms,
        "api.request"
    );

    response
}

async fn inject_request_id_into_error(
    response: Response<Body>,
    request_id: &str,
) -> Response<Body> {
    if !is_json_response(&response) || is_body_too_large(&response) {
        return response;
    }

    let (parts, body) = response.into_parts();
    let collected = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    if collected.len() > MAX_ERROR_BODY_BYTES {
        return Response::from_parts(parts, Body::from(collected));
    }

    let mut value: Value = match serde_json::from_slice(&collected) {
        Ok(value) => value,
        Err(_) => return Response::from_parts(parts, Body::from(collected)),
    };

    if let Some(object) = value.as_object_mut() {
        object
            .entry("request_id")
            .or_insert_with(|| Value::String(request_id.to_string()));
        let body = match serde_json::to_vec(&value) {
            Ok(body) => Body::from(body),
            Err(_) => Body::from(collected),
        };
        let mut response = Response::from_parts(parts, body);
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        return response;
    }

    Response::from_parts(parts, Body::from(collected))
}

fn extract_or_generate_request_id(req: &Request<Body>) -> String {
    if let Some(value) =
        header_value(req, REQUEST_ID_HEADER).or_else(|| header_value(req, CORRELATION_ID_HEADER))
    {
        return value;
    }

    Uuid::new_v4().to_string()
}

fn extract_or_generate_traceparent(req: &Request<Body>) -> TraceParent {
    if let Some(value) = header_value(req, TRACEPARENT_HEADER) {
        if let Some(parsed) = TraceParent::parse(&value) {
            return parsed;
        }
    }

    TraceParent::new()
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

fn is_json_response(response: &Response<Body>) -> bool {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"))
}

fn is_body_too_large(response: &Response<Body>) -> bool {
    response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|len| len > MAX_ERROR_BODY_BYTES)
}

impl TraceParent {
    pub fn new() -> Self {
        let trace_id = generate_nonzero_hex(16);
        let parent_id = generate_nonzero_hex(8);
        Self {
            version: "00".to_string(),
            trace_id,
            parent_id,
            flags: "00".to_string(),
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        let mut parts = trimmed.split('-');
        let version = parts.next()?.to_ascii_lowercase();
        let trace_id = parts.next()?.to_ascii_lowercase();
        let parent_id = parts.next()?.to_ascii_lowercase();
        let flags = parts.next()?.to_ascii_lowercase();

        if parts.next().is_some() {
            return None;
        }

        if version.len() != 2 || trace_id.len() != 32 || parent_id.len() != 16 || flags.len() != 2 {
            return None;
        }

        if !is_hex(&version) || !is_hex(&trace_id) || !is_hex(&parent_id) || !is_hex(&flags) {
            return None;
        }

        if is_all_zeros(&trace_id) || is_all_zeros(&parent_id) {
            return None;
        }

        Some(Self {
            version,
            trace_id,
            parent_id,
            flags,
        })
    }

    pub fn as_header_value(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.version, self.trace_id, self.parent_id, self.flags
        )
    }
}

impl Default for TraceParent {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_nonzero_hex(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    loop {
        let mut rng = rand::rngs::OsRng;
        rng.fill_bytes(&mut bytes);
        if bytes.iter().any(|b| *b != 0) {
            break;
        }
    }
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn is_hex(value: &str) -> bool {
    value.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_all_zeros(value: &str) -> bool {
    value.chars().all(|c| c == '0')
}
