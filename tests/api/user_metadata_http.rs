use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;
use uuid::Uuid;

use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_service::CreateRealmPayload;
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::permissions;

use crate::support::TestContext;

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json body")
}

async fn setup_realm(ctx: &TestContext) -> reauth::domain::realm::Realm {
    ctx.app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        })
        .await
        .expect("create realm")
}

async fn setup_user_admin_token(ctx: &TestContext, realm_id: Uuid) -> String {
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm_id,
            "metadata-admin",
            "password",
            Some("metadata-admin@example.com"),
            false,
        )
        .await
        .expect("create admin");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: "metadata-admin".to_string(),
                description: Some("Metadata admin".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm_id, role.id, permissions::USER_READ.to_string())
        .await
        .expect("assign user read");
    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm_id, role.id, permissions::USER_WRITE.to_string())
        .await
        .expect("assign user write");
    ctx.app_state
        .rbac_service
        .assign_role_to_user(realm_id, user.id, role.id)
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

async fn token_for_user(ctx: &TestContext, user: &reauth::domain::user::User) -> String {
    let (login, _) = ctx
        .app_state
        .auth_service
        .create_session(user, None, None, None)
        .await
        .expect("create session");
    login.access_token
}

fn request_with_json(
    method: &str,
    uri: String,
    token: Option<&str>,
    payload: serde_json::Value,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }
    builder
        .body(Body::from(payload.to_string()))
        .expect("json request")
}

fn request(method: &str, uri: String, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
    }
    builder.body(Body::empty()).expect("request")
}

#[tokio::test]
#[serial(test_db)]
async fn user_metadata_admin_and_self_endpoints_follow_visibility_rules() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let admin_token = setup_user_admin_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "metadata-target",
            "old-password-123",
            Some("metadata-target@example.com"),
            false,
        )
        .await
        .expect("create target user");
    let target_token = token_for_user(&ctx, &target_user).await;

    for (bucket, metadata) in [
        (
            "public",
            serde_json::json!({ "onboarding_completed": true, "theme": "dark" }),
        ),
        (
            "private",
            serde_json::json!({ "billing_sync_id": "bill_123", "risk_score": 7 }),
        ),
        ("unsafe", serde_json::json!({ "draft_display_name": "Ada" })),
    ] {
        let response = ctx
            .request(request_with_json(
                "PUT",
                format!(
                    "/api/realms/{}/users/{}/metadata/{}",
                    DEFAULT_REALM_NAME, target_user.id, bucket
                ),
                Some(&admin_token),
                serde_json::json!({ "metadata": metadata }),
            ))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    let admin_get = ctx
        .request(request(
            "GET",
            format!(
                "/api/realms/{}/users/{}/metadata",
                DEFAULT_REALM_NAME, target_user.id
            ),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(admin_get.status(), StatusCode::OK);
    let admin_json = json_body(admin_get).await;
    assert_eq!(admin_json["public_metadata"]["theme"], "dark");
    assert_eq!(
        admin_json["private_metadata"]["billing_sync_id"],
        "bill_123"
    );
    assert_eq!(admin_json["unsafe_metadata"]["draft_display_name"], "Ada");

    let admin_detail = ctx
        .request(request(
            "GET",
            format!(
                "/api/realms/{}/users/{}",
                DEFAULT_REALM_NAME, target_user.id
            ),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(admin_detail.status(), StatusCode::OK);
    let admin_detail_json = json_body(admin_detail).await;
    assert_eq!(admin_detail_json["public_metadata"]["theme"], "dark");
    assert_eq!(
        admin_detail_json["private_metadata"]["billing_sync_id"],
        "bill_123"
    );
    assert_eq!(
        admin_detail_json["unsafe_metadata"]["draft_display_name"],
        "Ada"
    );

    let self_get = ctx
        .request(request(
            "GET",
            "/api/realms/master/users/me/metadata".to_string(),
            Some(&target_token),
        ))
        .await;
    assert_eq!(self_get.status(), StatusCode::OK);
    let self_json = json_body(self_get).await;
    assert_eq!(self_json["public_metadata"]["theme"], "dark");
    assert_eq!(self_json["unsafe_metadata"]["draft_display_name"], "Ada");
    assert!(self_json.get("private_metadata").is_none());

    let self_detail = ctx
        .request(request(
            "GET",
            "/api/realms/master/users/me".to_string(),
            Some(&target_token),
        ))
        .await;
    assert_eq!(self_detail.status(), StatusCode::OK);
    let self_detail_json = json_body(self_detail).await;
    assert_eq!(self_detail_json["public_metadata"]["theme"], "dark");
    assert_eq!(
        self_detail_json["unsafe_metadata"]["draft_display_name"],
        "Ada"
    );
    assert!(self_detail_json.get("private_metadata").is_none());

    let self_update = ctx
        .request(request_with_json(
            "PUT",
            "/api/realms/master/users/me/metadata/unsafe".to_string(),
            Some(&target_token),
            serde_json::json!({ "metadata": { "draft_display_name": "Grace" } }),
        ))
        .await;
    assert_eq!(self_update.status(), StatusCode::OK);
    let self_update_json = json_body(self_update).await;
    assert_eq!(
        self_update_json["unsafe_metadata"]["draft_display_name"],
        "Grace"
    );
    assert!(self_update_json.get("private_metadata").is_none());

    let invalid = ctx
        .request(request_with_json(
            "PUT",
            format!(
                "/api/realms/{}/users/{}/metadata/public",
                DEFAULT_REALM_NAME, target_user.id
            ),
            Some(&admin_token),
            serde_json::json!({ "metadata": ["not", "an", "object"] }),
        ))
        .await;
    assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let invalid_json = json_body(invalid).await;
    assert_eq!(
        invalid_json["fields"]["metadata"],
        "Metadata must be a JSON object."
    );

    let anonymous = ctx
        .request(request(
            "GET",
            "/api/realms/master/users/me/metadata".to_string(),
            None,
        ))
        .await;
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
}
