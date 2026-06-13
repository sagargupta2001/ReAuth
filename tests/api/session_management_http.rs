use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;
use uuid::Uuid;

use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_service::CreateRealmPayload;
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::permissions;
use reauth::error::Error;

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

async fn setup_realm(ctx: &TestContext, name: &str) -> reauth::domain::realm::Realm {
    ctx.app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: name.to_string(),
        })
        .await
        .expect("create realm")
}

/// Create an admin user with the given permissions, returning (access_token, current_session_id).
async fn admin_with_permissions(
    ctx: &TestContext,
    realm_id: Uuid,
    username: &str,
    perms: &[&str],
) -> (String, Uuid) {
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
        .expect("create admin");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: username.to_string(),
                description: Some("session test role".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    for perm in perms {
        ctx.app_state
            .rbac_service
            .assign_permission_to_role(realm_id, role.id, perm.to_string())
            .await
            .expect("assign permission");
    }
    ctx.app_state
        .rbac_service
        .assign_role_to_user(realm_id, user.id, role.id)
        .await
        .expect("assign role");

    let (login, refresh) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");

    (login.access_token, refresh.id)
}

/// Create a plain target user with `count` active sessions. Returns (user_id, session_ids).
async fn user_with_sessions(
    ctx: &TestContext,
    realm_id: Uuid,
    username: &str,
    count: usize,
) -> (Uuid, Vec<Uuid>) {
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
        .expect("create target");

    let mut ids = Vec::new();
    for i in 0..count {
        // Distinct client_id per session so create_session does not revoke siblings.
        let (_, refresh) = ctx
            .app_state
            .auth_service
            .create_session(&user, Some(format!("client-{i}")), None, None)
            .await
            .expect("create target session");
        ids.push(refresh.id);
    }
    (user.id, ids)
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

fn empty_request(method: &str, uri: String, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("request")
}

async fn is_active(ctx: &TestContext, id: &Uuid) -> bool {
    ctx.app_state
        .session_repo
        .find_by_id(id)
        .await
        .expect("find_by_id")
        .is_some()
}

#[tokio::test]
#[serial(test_db)]
async fn list_requires_session_read_permission() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;

    let (no_read, _) =
        admin_with_permissions(&ctx, realm.id, "no-read", &[permissions::USER_WRITE]).await;
    let (reader, _) =
        admin_with_permissions(&ctx, realm.id, "reader", &[permissions::SESSION_READ]).await;

    let uri = format!("/api/realms/{}/sessions", DEFAULT_REALM_NAME);

    let forbidden = ctx
        .request(empty_request("GET", uri.clone(), &no_read))
        .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let ok = ctx.request(empty_request("GET", uri, &reader)).await;
    assert_eq!(ok.status(), StatusCode::OK);
}

#[tokio::test]
#[serial(test_db)]
async fn bulk_revoke_selected_revokes_targets_and_excludes_caller() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;

    let (admin, admin_sid) = admin_with_permissions(
        &ctx,
        realm.id,
        "admin",
        &[permissions::SESSION_READ, permissions::SESSION_REVOKE],
    )
    .await;

    let (_user, ids) = user_with_sessions(&ctx, realm.id, "target", 3).await;

    // Include the caller's own session id in the request; it must be excluded.
    let mut session_ids: Vec<Uuid> = ids.clone();
    session_ids.push(admin_sid);

    let res = ctx
        .request(json_request(
            "POST",
            format!("/api/realms/{}/sessions/revoke", DEFAULT_REALM_NAME),
            &admin,
            serde_json::json!({ "scope": "selected", "session_ids": session_ids }),
        ))
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["count"].as_u64(), Some(3));

    for id in &ids {
        assert!(
            !is_active(&ctx, id).await,
            "target session should be revoked"
        );
    }
    assert!(is_active(&ctx, &admin_sid).await, "caller stays active");
}

#[tokio::test]
#[serial(test_db)]
async fn bulk_revoke_empty_list_is_bad_request() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;
    let (admin, _) =
        admin_with_permissions(&ctx, realm.id, "admin", &[permissions::SESSION_REVOKE]).await;

    let res = ctx
        .request(json_request(
            "POST",
            format!("/api/realms/{}/sessions/revoke", DEFAULT_REALM_NAME),
            &admin,
            serde_json::json!({ "scope": "selected", "session_ids": [] }),
        ))
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial(test_db)]
async fn revoke_others_keeps_current_session() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;

    // The admin user is the caller. Give them extra sessions of their own.
    let (admin, admin_sid) =
        admin_with_permissions(&ctx, realm.id, "admin", &[permissions::SESSION_REVOKE]).await;
    let admin_user = ctx
        .app_state
        .user_service
        .find_by_username(&realm.id, "admin")
        .await
        .expect("lookup admin")
        .expect("admin exists");

    let mut other_ids = Vec::new();
    for i in 0..2 {
        let (_, refresh) = ctx
            .app_state
            .auth_service
            .create_session(&admin_user, Some(format!("c-{i}")), None, None)
            .await
            .expect("extra session");
        other_ids.push(refresh.id);
    }

    let res = ctx
        .request(json_request(
            "POST",
            format!("/api/realms/{}/sessions/revoke", DEFAULT_REALM_NAME),
            &admin,
            serde_json::json!({ "scope": "others" }),
        ))
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["count"].as_u64(), Some(2));

    assert!(
        is_active(&ctx, &admin_sid).await,
        "current session preserved"
    );
    for id in &other_ids {
        assert!(!is_active(&ctx, id).await, "other session revoked");
    }
}

#[tokio::test]
#[serial(test_db)]
async fn revoke_user_scope_requires_user_write() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;

    let (revoke_only, _) =
        admin_with_permissions(&ctx, realm.id, "revoker", &[permissions::SESSION_REVOKE]).await;
    let (revoke_and_write, _) = admin_with_permissions(
        &ctx,
        realm.id,
        "manager",
        &[permissions::SESSION_REVOKE, permissions::USER_WRITE],
    )
    .await;

    let (target_id, ids) = user_with_sessions(&ctx, realm.id, "victim", 2).await;
    let uri = format!("/api/realms/{}/sessions/revoke", DEFAULT_REALM_NAME);
    let payload = serde_json::json!({ "scope": "user", "user_id": target_id });

    // session:revoke alone is forbidden for whole-account eviction.
    let forbidden = ctx
        .request(json_request(
            "POST",
            uri.clone(),
            &revoke_only,
            payload.clone(),
        ))
        .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    for id in &ids {
        assert!(
            is_active(&ctx, id).await,
            "still active after forbidden call"
        );
    }

    // session:revoke + user:write succeeds.
    let ok = ctx
        .request(json_request("POST", uri, &revoke_and_write, payload))
        .await;
    assert_eq!(ok.status(), StatusCode::OK);
    let body = json_body(ok).await;
    assert_eq!(body["count"].as_u64(), Some(2));
    for id in &ids {
        assert!(!is_active(&ctx, id).await, "all user sessions revoked");
    }
}

#[tokio::test]
#[serial(test_db)]
async fn step_up_forces_reauth_on_next_refresh() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;
    let (admin, _) =
        admin_with_permissions(&ctx, realm.id, "admin", &[permissions::SESSION_REVOKE]).await;

    let (_user, ids) = user_with_sessions(&ctx, realm.id, "target", 1).await;
    let target = ids[0];

    let res = ctx
        .request(empty_request(
            "POST",
            format!(
                "/api/realms/{}/sessions/{}/step-up",
                DEFAULT_REALM_NAME, target
            ),
            &admin,
        ))
        .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Silent refresh of a stepped-up session must be rejected.
    match ctx.app_state.auth_service.refresh_session(target).await {
        Err(Error::ReauthRequired) => {}
        Ok(_) => panic!("refresh should fail with ReauthRequired"),
        Err(other) => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
#[serial(test_db)]
async fn cannot_step_up_own_current_session() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx, DEFAULT_REALM_NAME).await;
    let (admin, admin_sid) =
        admin_with_permissions(&ctx, realm.id, "admin", &[permissions::SESSION_REVOKE]).await;

    let res = ctx
        .request(empty_request(
            "POST",
            format!(
                "/api/realms/{}/sessions/{}/step-up",
                DEFAULT_REALM_NAME, admin_sid
            ),
            &admin,
        ))
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(is_active(&ctx, &admin_sid).await);
}

#[tokio::test]
#[serial(test_db)]
async fn revoke_is_realm_scoped() {
    let ctx = TestContext::new().await;
    let realm_a = setup_realm(&ctx, DEFAULT_REALM_NAME).await;
    let realm_b = setup_realm(&ctx, "other-realm").await;

    let (admin_a, _) =
        admin_with_permissions(&ctx, realm_a.id, "admin-a", &[permissions::SESSION_REVOKE]).await;

    let (_user_b, ids_b) = user_with_sessions(&ctx, realm_b.id, "user-b", 1).await;
    let foreign = ids_b[0];

    // Targeting realm B's session through realm A revokes nothing.
    let res = ctx
        .request(json_request(
            "POST",
            format!("/api/realms/{}/sessions/revoke", DEFAULT_REALM_NAME),
            &admin_a,
            serde_json::json!({ "scope": "selected", "session_ids": [foreign] }),
        ))
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["count"].as_u64(), Some(0));
    assert!(
        is_active(&ctx, &foreign).await,
        "foreign realm session untouched"
    );
}
