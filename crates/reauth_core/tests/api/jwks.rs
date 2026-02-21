use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;

use crate::support::TestContext;

#[tokio::test]
#[serial(test_db)]
async fn jwks_endpoint_returns_keys() {
    let ctx = TestContext::new().await;

    let response = ctx
        .request(
            Request::builder()
                .uri("/api/realms/master/oidc/.well-known/jwks.json")
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

    let json: serde_json::Value = serde_json::from_slice(&body).expect("invalid jwks JSON");
    let keys = json
        .get("keys")
        .and_then(|value| value.as_array())
        .expect("jwks should include keys array");

    assert!(!keys.is_empty(), "jwks should contain at least one key");
}
