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

async fn setup_user_writer_token(ctx: &TestContext, realm_id: Uuid) -> String {
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm_id,
            "user-writer",
            "password",
            Some("writer@example.com"),
            false,
        )
        .await
        .expect("create writer");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: "user-writer".to_string(),
                description: Some("User writer".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

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

#[tokio::test]
#[serial(test_db)]
async fn user_credentials_list_and_password_update_work() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-user",
            "old-password-123",
            Some("target@example.com"),
            false,
        )
        .await
        .expect("create target user");
    let old_hash = target_user.hashed_password.clone();

    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("list request");
    let list_res = ctx.request(list_req).await;
    assert_eq!(list_res.status(), StatusCode::OK);
    let list_json = json_body(list_res).await;
    assert_eq!(
        list_json
            .get("password")
            .and_then(|value| value.get("configured"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        list_json
            .get("passkeys")
            .and_then(|value| value.as_array())
            .map(|arr| arr.len()),
        Some(0)
    );

    let update_req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/password",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "password": "new-password-123" }).to_string(),
        ))
        .expect("update request");
    let update_res = ctx.request(update_req).await;
    assert_eq!(update_res.status(), StatusCode::OK);

    let updated_user = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, target_user.id)
        .await
        .expect("updated user");
    assert_ne!(old_hash, updated_user.hashed_password);
}

#[tokio::test]
#[serial(test_db)]
async fn user_credentials_password_policy_controls_work() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-user-2",
            "old-password-123",
            Some("target-2@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let disable_without_policy_req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/password-policy",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "password_login_disabled": true }).to_string(),
        ))
        .expect("disable request");
    let disable_without_policy_res = ctx.request(disable_without_policy_req).await;
    assert_eq!(disable_without_policy_res.status(), StatusCode::BAD_REQUEST);

    let update_policy_req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/password-policy",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "force_reset_on_next_login": true,
            })
            .to_string(),
        ))
        .expect("update policy request");
    let update_policy_res = ctx.request(update_policy_req).await;
    assert_eq!(update_policy_res.status(), StatusCode::OK);

    let updated_user = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, target_user.id)
        .await
        .expect("updated user");
    assert!(updated_user.force_password_reset);
    assert!(!updated_user.password_login_disabled);

    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("list request");
    let list_res = ctx.request(list_req).await;
    assert_eq!(list_res.status(), StatusCode::OK);
    let list_json = json_body(list_res).await;
    assert_eq!(
        list_json
            .get("password")
            .and_then(|value| value.get("force_reset_on_next_login"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        list_json
            .get("password")
            .and_then(|value| value.get("password_login_disabled"))
            .and_then(|value| value.as_bool()),
        Some(false)
    );
}

#[tokio::test]
#[serial(test_db)]
async fn user_credentials_passkey_metadata_returns_not_found_for_unknown_credential() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-user-3",
            "old-password-123",
            Some("target-3@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/passkeys/{}",
            DEFAULT_REALM_NAME,
            target_user.id,
            Uuid::new_v4()
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "friendly_name": "Laptop Key" }).to_string(),
        ))
        .expect("request");
    let res = ctx.request(req).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
