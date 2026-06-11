use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;
use uuid::Uuid;

use reauth::application::idp_service::CreateIdentityProviderRequest;
use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_service::{CreateRealmPayload, UpdateRealmPayload};
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::identity_provider::{IdentityProviderProtocol, OAuthBrokerResult};
use reauth::domain::permissions;
use reauth::domain::user::User;

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

async fn link_federated_identity(
    ctx: &TestContext,
    realm_id: Uuid,
    user: &User,
    local_password: &str,
    alias: &str,
    subject: &str,
) -> Uuid {
    let provider = ctx
        .app_state
        .identity_provider_service
        .create(
            realm_id,
            CreateIdentityProviderRequest {
                preset: None,
                alias: alias.to_string(),
                display_name: format!("{} Login", alias),
                protocol: IdentityProviderProtocol::Oauth2,
                client_id: format!("client-{}", alias),
                client_secret: Some("secret".to_string()),
                issuer: None,
                authorization_endpoint: Some("https://example.com/oauth/authorize".to_string()),
                token_endpoint: Some("https://example.com/oauth/token".to_string()),
                userinfo_endpoint: Some("https://example.com/oauth/userinfo".to_string()),
                jwks_uri: None,
                scopes: Some(vec!["openid".to_string(), "email".to_string()]),
                claim_mapping: Some(serde_json::json!({})),
                pkce_required: Some(true),
                allow_login: Some(true),
                allow_link: Some(true),
                allow_jit_provisioning: Some(false),
                allow_email_auto_link: Some(false),
                require_verified_email: Some(true),
                icon_ref: None,
                button_color: None,
                sort_order: Some(0),
                enabled: Some(true),
            },
        )
        .await
        .expect("create provider");

    ctx.app_state
        .oauth_broker_service
        .complete_manual_link(
            realm_id,
            &OAuthBrokerResult {
                user_id: None,
                output: "link_required".to_string(),
                provider_id: provider.id,
                provider_alias: provider.alias.clone(),
                provider_display_name: provider.display_name.clone(),
                subject: subject.to_string(),
                external_email: None,
                external_username: Some(user.username.clone()),
                message: None,
            },
            &user.username,
            local_password,
        )
        .await
        .expect("link federated identity");

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm_id, user.id)
        .await
        .expect("list credentials");
    credentials
        .federated_identities
        .into_iter()
        .find(|identity| identity.subject == subject)
        .map(|identity| identity.id)
        .expect("linked federated identity")
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
    let (_target_login, target_refresh_token) = ctx
        .app_state
        .auth_service
        .create_session(&target_user, None, None, None)
        .await
        .expect("create target session");

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
            serde_json::json!({
                "password": "short",
                "skip_password_checks": true,
                "sign_out_all_sessions": true,
            })
            .to_string(),
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

    let revoked_refresh_token = ctx
        .app_state
        .session_repo
        .find_by_id_any(&target_refresh_token.id)
        .await
        .expect("find refresh token")
        .expect("refresh token exists");
    assert!(revoked_refresh_token.revoked_at.is_some());
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
async fn user_credentials_federated_identities_list_and_unlink_work() {
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

    let federated_identity_id = link_federated_identity(
        &ctx,
        realm.id,
        &target_user,
        "old-password-123",
        "github",
        "github-user-123",
    )
    .await;

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
            .get("federated_identities")
            .and_then(|value| value.as_array())
            .map(|arr| arr.len()),
        Some(1)
    );
    assert_eq!(
        list_json
            .get("federated_identities")
            .and_then(|value| value.as_array())
            .and_then(|arr| arr.first())
            .and_then(|value| value.get("provider_alias"))
            .and_then(|value| value.as_str()),
        Some("github")
    );
    assert_eq!(
        list_json
            .get("federated_identities")
            .and_then(|value| value.as_array())
            .and_then(|arr| arr.first())
            .and_then(|value| value.get("linked_via"))
            .and_then(|value| value.as_str()),
        Some("manual")
    );

    let unlink_req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/federated/{}",
            DEFAULT_REALM_NAME, target_user.id, federated_identity_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("unlink request");
    let unlink_res = ctx.request(unlink_req).await;
    assert_eq!(unlink_res.status(), StatusCode::OK);

    let updated_credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, target_user.id)
        .await
        .expect("updated credentials");
    assert!(updated_credentials.federated_identities.is_empty());
}

#[tokio::test]
#[serial(test_db)]
async fn user_credentials_unlink_last_federated_factor_returns_conflict() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-user-4",
            "old-password-123",
            Some("target-4@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let federated_identity_id = link_federated_identity(
        &ctx,
        realm.id,
        &target_user,
        "old-password-123",
        "google",
        "google-user-123",
    )
    .await;

    ctx.app_state
        .user_service
        .update_credential_policy(realm.id, target_user.id, None, Some(true))
        .await
        .expect("disable password login directly for seed state");

    let unlink_req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/federated/{}",
            DEFAULT_REALM_NAME, target_user.id, federated_identity_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("unlink request");
    let unlink_res = ctx.request(unlink_req).await;
    assert_eq!(unlink_res.status(), StatusCode::CONFLICT);
    let unlink_json = json_body(unlink_res).await;
    assert_eq!(
        unlink_json.get("code").and_then(|value| value.as_str()),
        Some("request.conflict")
    );
    assert_eq!(
        unlink_json.get("error").and_then(|value| value.as_str()),
        Some(
            "Cannot unlink the last sign-in method for this user. Configure a password or passkey first."
        )
    );

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, target_user.id)
        .await
        .expect("credentials after failed unlink");
    assert_eq!(credentials.federated_identities.len(), 1);
}

#[tokio::test]
#[serial(test_db)]
async fn user_credentials_can_unlink_last_federated_factor_when_guard_disabled() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    ctx.app_state
        .realm_service
        .update_realm(
            realm.id,
            UpdateRealmPayload {
                name: None,
                access_token_ttl_secs: None,
                refresh_token_ttl_secs: None,
                pkce_required_public_clients: None,
                lockout_threshold: None,
                lockout_duration_secs: None,
                registration_enabled: None,
                default_registration_role_ids: None,
                invitation_resend_limit: None,
                idp_broker_enabled: None,
                idp_default_jit_policy: None,
                idp_default_email_link_policy: None,
                idp_minimum_remaining_factor: Some(false),
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
                invitation_flow_id: None,
            },
        )
        .await
        .expect("disable last factor guard");

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-user-5",
            "temporary-password-123",
            Some("target-5@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let federated_identity_id = link_federated_identity(
        &ctx,
        realm.id,
        &target_user,
        "temporary-password-123",
        "github",
        "github-user-789",
    )
    .await;

    ctx.app_state
        .user_service
        .update_credential_policy(realm.id, target_user.id, None, Some(true))
        .await
        .expect("disable password login");

    let unlink_req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/users/{}/credentials/federated/{}",
            DEFAULT_REALM_NAME, target_user.id, federated_identity_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("unlink request");
    let unlink_res = ctx.request(unlink_req).await;
    assert_eq!(unlink_res.status(), StatusCode::OK);

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, target_user.id)
        .await
        .expect("credentials after unlink");
    assert!(credentials.federated_identities.is_empty());
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
            "target-user-5",
            "old-password-123",
            Some("target-5@example.com"),
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
