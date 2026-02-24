use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use serial_test::serial;

use reauth_core::application::rbac_service::CreateRolePayload;
use reauth_core::application::realm_service::CreateRealmPayload;
use reauth_core::constants::DEFAULT_REALM_NAME;
use reauth_core::domain::permissions;

use crate::support::TestContext;

async fn setup_observability_user(ctx: &TestContext) -> String {
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        })
        .await
        .expect("create realm");

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "observer", "password")
        .await
        .expect("create user");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm.id,
            CreateRolePayload {
                name: "observability-reader".to_string(),
                description: Some("Observability reader".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm.id, role.id, permissions::EVENT_READ.to_string())
        .await
        .expect("assign permission");

    ctx.app_state
        .rbac_service
        .assign_role_to_user(realm.id, user.id, role.id)
        .await
        .expect("assign role");

    let (login, _) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");

    login.access_token
}

#[tokio::test]
#[serial(test_db)]
async fn observability_endpoints_return_ok() {
    let ctx = TestContext::new().await;
    let token = setup_observability_user(&ctx).await;

    let logs_request = Request::builder()
        .uri("/api/system/observability/logs")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("logs request");
    let logs_response = ctx.request(logs_request).await;
    assert_eq!(logs_response.status(), StatusCode::OK);
    let logs_body = logs_response
        .into_body()
        .collect()
        .await
        .expect("logs body")
        .to_bytes();
    let logs_json: Value = serde_json::from_slice(&logs_body).expect("logs json");
    assert!(logs_json
        .get("data")
        .and_then(|value| value.as_array())
        .is_some());
    assert!(logs_json.get("meta").is_some());

    let traces_request = Request::builder()
        .uri("/api/system/observability/traces")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("traces request");
    let traces_response = ctx.request(traces_request).await;
    assert_eq!(traces_response.status(), StatusCode::OK);
    let traces_body = traces_response
        .into_body()
        .collect()
        .await
        .expect("traces body")
        .to_bytes();
    let traces_json: Value = serde_json::from_slice(&traces_body).expect("traces json");
    assert!(traces_json
        .get("data")
        .and_then(|value| value.as_array())
        .is_some());
    assert!(traces_json.get("meta").is_some());

    let trace_spans_request = Request::builder()
        .uri("/api/system/observability/traces/test-trace-id")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("trace spans request");
    let trace_spans_response = ctx.request(trace_spans_request).await;
    assert_eq!(trace_spans_response.status(), StatusCode::OK);
    let trace_spans_body = trace_spans_response
        .into_body()
        .collect()
        .await
        .expect("trace spans body")
        .to_bytes();
    let trace_spans_json: Value =
        serde_json::from_slice(&trace_spans_body).expect("trace spans json");
    assert!(trace_spans_json.is_array());

    let cache_stats_request = Request::builder()
        .uri("/api/system/observability/cache/stats")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("cache stats request");
    let cache_stats_response = ctx.request(cache_stats_request).await;
    assert_eq!(cache_stats_response.status(), StatusCode::OK);
    let cache_stats_body = cache_stats_response
        .into_body()
        .collect()
        .await
        .expect("cache stats body")
        .to_bytes();
    let cache_stats_json: Value = serde_json::from_slice(&cache_stats_body).expect("cache json");
    assert!(cache_stats_json.is_array());
    let first = cache_stats_json
        .as_array()
        .and_then(|values| values.first())
        .expect("cache stats entry");
    assert!(first.get("namespace").is_some());
    assert!(first.get("hit_rate").is_some());
    assert!(first.get("entry_count").is_some());
    assert!(first.get("max_capacity").is_some());

    let cache_stats_namespace_request = Request::builder()
        .uri("/api/system/observability/cache/stats?namespace=user_permissions")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("cache stats namespace request");
    let cache_stats_namespace_response = ctx.request(cache_stats_namespace_request).await;
    assert_eq!(cache_stats_namespace_response.status(), StatusCode::OK);
    let cache_stats_namespace_body = cache_stats_namespace_response
        .into_body()
        .collect()
        .await
        .expect("cache stats namespace body")
        .to_bytes();
    let cache_stats_namespace_json: Value =
        serde_json::from_slice(&cache_stats_namespace_body).expect("cache json");
    assert_eq!(
        cache_stats_namespace_json
            .get("namespace")
            .and_then(|value| value.as_str()),
        Some("user_permissions")
    );

    let cache_flush_request = Request::builder()
        .uri("/api/system/observability/cache/flush")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"namespace":"all"}"#))
        .expect("cache flush request");
    let cache_flush_response = ctx.request(cache_flush_request).await;
    assert_eq!(cache_flush_response.status(), StatusCode::OK);
    let cache_flush_body = cache_flush_response
        .into_body()
        .collect()
        .await
        .expect("cache flush body")
        .to_bytes();
    let cache_flush_json: Value = serde_json::from_slice(&cache_flush_body).expect("flush json");
    assert_eq!(
        cache_flush_json
            .get("flushed")
            .and_then(|value| value.as_str()),
        Some("all")
    );

    let clear_logs_request = Request::builder()
        .uri("/api/system/observability/logs/clear")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("clear logs request");
    let clear_logs_response = ctx.request(clear_logs_request).await;
    assert_eq!(clear_logs_response.status(), StatusCode::OK);
    let clear_logs_body = clear_logs_response
        .into_body()
        .collect()
        .await
        .expect("clear logs body")
        .to_bytes();
    let clear_logs_json: Value = serde_json::from_slice(&clear_logs_body).expect("clear logs json");
    assert!(clear_logs_json.get("deleted").is_some());

    let clear_traces_request = Request::builder()
        .uri("/api/system/observability/traces/clear")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("clear traces request");
    let clear_traces_response = ctx.request(clear_traces_request).await;
    assert_eq!(clear_traces_response.status(), StatusCode::OK);
    let clear_traces_body = clear_traces_response
        .into_body()
        .collect()
        .await
        .expect("clear traces body")
        .to_bytes();
    let clear_traces_json: Value =
        serde_json::from_slice(&clear_traces_body).expect("clear traces json");
    assert!(clear_traces_json.get("deleted").is_some());

    let metrics_request = Request::builder()
        .uri("/api/system/observability/metrics")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("metrics request");
    let metrics_response = ctx.request(metrics_request).await;
    assert_eq!(metrics_response.status(), StatusCode::OK);
    let metrics_body = metrics_response
        .into_body()
        .collect()
        .await
        .expect("metrics body")
        .to_bytes();
    let metrics_json: Value = serde_json::from_slice(&metrics_body).expect("metrics json");
    assert!(metrics_json.get("request_count").is_some());
    assert!(metrics_json.get("latency_ms").is_some());
}
