use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;

use crate::support::TestContext;

#[tokio::test]
#[serial(test_db)]
async fn health_returns_ok() {
    let ctx = TestContext::new().await;

    let response = ctx
        .request(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body read failed")
        .to_bytes();

    assert_eq!(body, "OK");
}
