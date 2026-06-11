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

async fn setup_token_with_permission(
    ctx: &TestContext,
    realm_id: Uuid,
    username: &str,
    permission: &str,
) -> String {
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm_id,
            username,
            "password",
            Some(&format!("{username}@example.com")),
            false,
        )
        .await
        .expect("create actor");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: username.to_string(),
                description: Some(format!("{permission} test role")),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm_id, role.id, permission.to_string())
        .await
        .expect("assign permission");
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

fn empty_request(method: &str, uri: String, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("request")
}

fn json_request(
    method: &str,
    uri: String,
    token: &str,
    payload: serde_json::Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("json request")
}

#[tokio::test]
#[serial(test_db)]
async fn user_admin_lock_ban_and_delete_actions_work() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let lock_token =
        setup_token_with_permission(&ctx, realm.id, "user-locker", permissions::USER_LOCK).await;
    let ban_token =
        setup_token_with_permission(&ctx, realm.id, "user-banner", permissions::USER_BAN).await;
    let delete_token =
        setup_token_with_permission(&ctx, realm.id, "user-deleter", permissions::USER_DELETE).await;

    let lock_target = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "lock-target",
            "old-password-123",
            Some("lock-target@example.com"),
            false,
        )
        .await
        .expect("create lock target");
    let (_, lock_target_refresh) = ctx
        .app_state
        .auth_service
        .create_session(&lock_target, None, None, None)
        .await
        .expect("create lock target session");

    let lock_res = ctx
        .request(empty_request(
            "POST",
            format!(
                "/api/realms/{}/users/{}/lock",
                DEFAULT_REALM_NAME, lock_target.id
            ),
            &lock_token,
        ))
        .await;
    assert_eq!(lock_res.status(), StatusCode::OK);
    let lock_json = json_body(lock_res).await;
    assert!(lock_json["locked_until"].as_str().is_some());

    let locked_user = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, lock_target.id)
        .await
        .expect("locked user");
    assert!(locked_user.locked_until.is_some());
    let revoked_refresh = ctx
        .app_state
        .session_repo
        .find_by_id_any(&lock_target_refresh.id)
        .await
        .expect("find refresh")
        .expect("refresh exists");
    assert!(revoked_refresh.revoked_at.is_some());

    let ban_target = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "ban-target",
            "old-password-123",
            Some("ban-target@example.com"),
            false,
        )
        .await
        .expect("create ban target");
    let (_, ban_target_refresh) = ctx
        .app_state
        .auth_service
        .create_session(&ban_target, None, None, None)
        .await
        .expect("create ban target session");

    let ban_res = ctx
        .request(empty_request(
            "POST",
            format!(
                "/api/realms/{}/users/{}/ban",
                DEFAULT_REALM_NAME, ban_target.id
            ),
            &ban_token,
        ))
        .await;
    assert_eq!(ban_res.status(), StatusCode::OK);
    let ban_json = json_body(ban_res).await;
    assert!(ban_json["banned_at"].as_str().is_some());

    let banned_user = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, ban_target.id)
        .await
        .expect("banned user");
    assert!(banned_user.banned_at.is_some());
    let revoked_refresh = ctx
        .app_state
        .session_repo
        .find_by_id_any(&ban_target_refresh.id)
        .await
        .expect("find refresh")
        .expect("refresh exists");
    assert!(revoked_refresh.revoked_at.is_some());

    let delete_target = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "delete-target",
            "old-password-123",
            Some("delete-target@example.com"),
            false,
        )
        .await
        .expect("create delete target");

    let delete_res = ctx
        .request(json_request(
            "DELETE",
            format!("/api/realms/{}/users", DEFAULT_REALM_NAME),
            &delete_token,
            serde_json::json!({ "user_ids": [delete_target.id] }),
        ))
        .await;
    assert_eq!(delete_res.status(), StatusCode::OK);
    let delete_json = json_body(delete_res).await;
    assert_eq!(delete_json["count"].as_u64(), Some(1));

    let deleted = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, delete_target.id)
        .await;
    assert!(deleted.is_err());
}

#[tokio::test]
#[serial(test_db)]
async fn user_write_permission_cannot_delete_lock_or_ban_users() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let write_token =
        setup_token_with_permission(&ctx, realm.id, "user-writer", permissions::USER_WRITE).await;

    let target = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "action-target",
            "old-password-123",
            Some("action-target@example.com"),
            false,
        )
        .await
        .expect("create target");

    for (method, uri) in [
        (
            "POST",
            format!(
                "/api/realms/{}/users/{}/lock",
                DEFAULT_REALM_NAME, target.id
            ),
        ),
        (
            "POST",
            format!("/api/realms/{}/users/{}/ban", DEFAULT_REALM_NAME, target.id),
        ),
    ] {
        let res = ctx.request(empty_request(method, uri, &write_token)).await;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    let delete_res = ctx
        .request(json_request(
            "DELETE",
            format!("/api/realms/{}/users", DEFAULT_REALM_NAME),
            &write_token,
            serde_json::json!({ "user_ids": [target.id] }),
        ))
        .await;
    assert_eq!(delete_res.status(), StatusCode::FORBIDDEN);
}
