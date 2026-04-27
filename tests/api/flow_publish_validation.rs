use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;
use uuid::Uuid;

use reauth::application::flow_manager::UpdateDraftRequest;
use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_service::CreateRealmPayload;
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::permissions;

use crate::support::TestContext;

async fn setup_realm_admin(ctx: &TestContext) -> (reauth::domain::realm::Realm, String) {
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
        .create_user(realm.id, "flow-admin", "password", None)
        .await
        .expect("create user");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm.id,
            CreateRolePayload {
                name: "flow-admin".to_string(),
                description: Some("Flow admin".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm.id, role.id, permissions::REALM_WRITE.to_string())
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

    (realm, login.access_token)
}

#[tokio::test]
#[serial(test_db)]
async fn publish_flow_rejects_missing_theme_page_binding() {
    let ctx = TestContext::new().await;
    let (realm, token) = setup_realm_admin(&ctx).await;

    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "password", "type": "core.auth.password", "data": { "config": { "ui": { "page_key": "custom.missing" } } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-password", "source": "start", "target": "password", "sourceHandle": "next" },
            { "id": "e-password-allow", "source": "password", "target": "allow", "sourceHandle": "success" }
        ]
    });

    ctx.app_state
        .flow_manager
        .update_draft(
            flow_id,
            UpdateDraftRequest {
                name: None,
                description: None,
                graph_json: Some(graph),
            },
        )
        .await
        .expect("update draft");

    let request = Request::builder()
        .uri(format!(
            "/api/realms/{}/flows/{}/publish",
            realm.name, flow_id
        ))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("publish request");

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("error json");
    let message = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(message.contains("Missing theme pages"));
    let issues = json
        .get("details")
        .and_then(|v| v.get("issues"))
        .and_then(|v| v.as_array())
        .expect("issues array");
    assert!(issues.iter().any(|issue| {
        issue
            .get("node_ids")
            .and_then(|v| v.as_array())
            .map(|ids| ids.iter().any(|id| id == "password"))
            .unwrap_or(false)
    }));
}

#[tokio::test]
#[serial(test_db)]
async fn publish_flow_rejects_category_mismatch() {
    let ctx = TestContext::new().await;
    let (realm, token) = setup_realm_admin(&ctx).await;

    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "consent", "type": "core.oidc.consent", "data": { "config": { "ui": { "page_key": "login" } } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-consent", "source": "start", "target": "consent", "sourceHandle": "next" },
            { "id": "e-consent-allow", "source": "consent", "target": "allow", "sourceHandle": "success" }
        ]
    });

    ctx.app_state
        .flow_manager
        .update_draft(
            flow_id,
            UpdateDraftRequest {
                name: None,
                description: None,
                graph_json: Some(graph),
            },
        )
        .await
        .expect("update draft");

    let request = Request::builder()
        .uri(format!(
            "/api/realms/{}/flows/{}/publish",
            realm.name, flow_id
        ))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("publish request");

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("error json");
    let message = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(message.contains("Page category mismatches"));
    let issues = json
        .get("details")
        .and_then(|v| v.get("issues"))
        .and_then(|v| v.as_array())
        .expect("issues array");
    assert!(issues.iter().any(|issue| {
        issue
            .get("node_ids")
            .and_then(|v| v.as_array())
            .map(|ids| ids.iter().any(|id| id == "consent"))
            .unwrap_or(false)
    }));
}
