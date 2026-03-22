use crate::domain::realm_security_headers::RealmSecurityHeaders;
use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn attach_realm_security_headers(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let realm_id = extract_realm_id(req.uri().path(), &state).await;
    let mut response = next.run(req).await;

    if let Some(realm_id) = realm_id {
        if let Ok(settings) = state
            .realm_security_headers_service
            .get_settings(realm_id)
            .await
        {
            apply_security_headers(&mut response, &settings);
        }
    }

    response
}

async fn extract_realm_id(path: &str, state: &AppState) -> Option<Uuid> {
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    if segments.next()? != "api" {
        return None;
    }
    if segments.next()? != "realms" {
        return None;
    }
    let realm_segment = segments.next()?;
    if let Ok(id) = Uuid::parse_str(realm_segment) {
        return Some(id);
    }
    let realm = state
        .realm_service
        .find_by_name(realm_segment)
        .await
        .ok()
        .flatten()?;
    Some(realm.id)
}

fn apply_security_headers(response: &mut Response, settings: &RealmSecurityHeaders) {
    let headers = response.headers_mut();
    insert_header(
        headers,
        header::HeaderName::from_static("x-frame-options"),
        settings.x_frame_options.as_deref(),
    );
    insert_header(
        headers,
        header::HeaderName::from_static("content-security-policy"),
        settings.content_security_policy.as_deref(),
    );
    insert_header(
        headers,
        header::HeaderName::from_static("x-content-type-options"),
        settings.x_content_type_options.as_deref(),
    );
    insert_header(
        headers,
        header::HeaderName::from_static("referrer-policy"),
        settings.referrer_policy.as_deref(),
    );
    insert_header(
        headers,
        header::HeaderName::from_static("strict-transport-security"),
        settings.strict_transport_security.as_deref(),
    );
}

fn insert_header(
    headers: &mut axum::http::HeaderMap,
    name: header::HeaderName,
    value: Option<&str>,
) {
    let Some(value) = value else {
        return;
    };
    if value.trim().is_empty() {
        return;
    }
    match HeaderValue::from_str(value) {
        Ok(header_value) => {
            headers.insert(name, header_value);
        }
        Err(err) => {
            tracing::warn!(
                "Invalid security header value for {}: {}",
                name.as_str(),
                err
            );
        }
    }
}
