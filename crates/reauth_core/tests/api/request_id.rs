use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use serial_test::serial;

use crate::support::TestContext;

#[tokio::test]
#[serial(test_db)]
async fn request_id_is_generated_and_returned() {
    let ctx = TestContext::new().await;

    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let header = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id");
    let value = header.to_str().expect("request id string");
    assert!(!value.trim().is_empty());
}

#[tokio::test]
#[serial(test_db)]
async fn request_id_is_echoed_when_provided() {
    let ctx = TestContext::new().await;
    let request_id = "test-request-id-123";

    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .header("x-request-id", request_id)
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let header = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id");
    let value = header.to_str().expect("request id string");
    assert_eq!(value, request_id);
}

#[tokio::test]
#[serial(test_db)]
async fn correlation_id_is_accepted_and_propagated() {
    let ctx = TestContext::new().await;
    let correlation_id = "corr-abc-123";

    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .header("x-correlation-id", correlation_id)
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let request_id_value = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id")
        .to_str()
        .expect("request id string")
        .to_string();
    assert_eq!(request_id_value, correlation_id);

    let correlation = response
        .headers()
        .get("x-correlation-id")
        .expect("x-correlation-id");
    let correlation_value = correlation.to_str().expect("correlation id string");
    assert_eq!(correlation_value, correlation_id);
}

#[tokio::test]
#[serial(test_db)]
async fn traceparent_is_echoed_when_provided() {
    let ctx = TestContext::new().await;
    let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .header("traceparent", traceparent)
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("traceparent").expect("traceparent");
    let value = header.to_str().expect("traceparent string");
    assert_eq!(value, traceparent);
}

#[tokio::test]
#[serial(test_db)]
async fn traceparent_is_generated_when_missing() {
    let ctx = TestContext::new().await;

    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers().get("traceparent").expect("traceparent");
    let value = header.to_str().expect("traceparent string");
    assert!(is_valid_traceparent(value));
}

#[tokio::test]
#[serial(test_db)]
async fn error_response_includes_request_id() {
    let ctx = TestContext::new().await;

    let request = Request::builder()
        .uri("/api/realms")
        .method("GET")
        .body(Body::empty())
        .expect("request");

    let response = ctx.request(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let request_id_value = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id")
        .to_str()
        .expect("request id string")
        .to_string();

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body read failed")
        .to_bytes();
    let value: Value = serde_json::from_slice(&body).expect("json body");
    let request_id_body = value
        .get("request_id")
        .and_then(|value| value.as_str())
        .expect("request_id field");
    assert_eq!(request_id_body, request_id_value);
}

fn is_valid_traceparent(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 4 {
        return false;
    }
    if parts[0].len() != 2 || parts[1].len() != 32 || parts[2].len() != 16 || parts[3].len() != 2 {
        return false;
    }
    if !parts
        .iter()
        .all(|part| part.chars().all(|c| c.is_ascii_hexdigit()))
    {
        return false;
    }
    if parts[1].chars().all(|c| c == '0') || parts[2].chars().all(|c| c == '0') {
        return false;
    }
    true
}
