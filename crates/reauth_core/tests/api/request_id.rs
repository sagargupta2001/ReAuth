use axum::body::Body;
use axum::http::{Request, StatusCode};
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
    let request_id = response
        .headers()
        .get("x-request-id")
        .expect("x-request-id");
    let request_id_value = request_id.to_str().expect("request id string");
    assert_eq!(request_id_value, correlation_id);

    let correlation = response
        .headers()
        .get("x-correlation-id")
        .expect("x-correlation-id");
    let correlation_value = correlation.to_str().expect("correlation id string");
    assert_eq!(correlation_value, correlation_id);
}
