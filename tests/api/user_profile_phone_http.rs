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
            "profile-writer",
            "password",
            Some("profile-writer@example.com"),
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
                name: "profile-writer".to_string(),
                description: Some("Profile writer".to_string()),
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
async fn user_profile_and_phone_number_admin_endpoints_work() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_user_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "target-profile",
            "old-password-123",
            Some("target-profile@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let update_profile_req = json_request(
        "PUT",
        format!(
            "/api/realms/{}/users/{}",
            DEFAULT_REALM_NAME, target_user.id
        ),
        &token,
        serde_json::json!({
            "username": "target-renamed",
            "first_name": "Ada",
            "last_name": "Lovelace"
        }),
    );
    let update_profile_res = ctx.request(update_profile_req).await;
    assert_eq!(update_profile_res.status(), StatusCode::OK);
    let update_profile_json = json_body(update_profile_res).await;
    assert_eq!(update_profile_json["username"], "target-renamed");
    assert_eq!(update_profile_json["first_name"], "Ada");
    assert_eq!(update_profile_json["last_name"], "Lovelace");
    assert!(update_profile_json["updated_at"].as_str().is_some());

    let add_primary_req = json_request(
        "POST",
        format!(
            "/api/realms/{}/users/{}/phone-numbers",
            DEFAULT_REALM_NAME, target_user.id
        ),
        &token,
        serde_json::json!({
            "phone_number": "+1 (555) 0100",
            "is_primary": true,
            "is_verified": false
        }),
    );
    let add_primary_res = ctx.request(add_primary_req).await;
    assert_eq!(add_primary_res.status(), StatusCode::CREATED);
    let primary_json = json_body(add_primary_res).await;
    let primary_id = primary_json["id"].as_str().expect("primary id").to_string();

    let add_second_req = json_request(
        "POST",
        format!(
            "/api/realms/{}/users/{}/phone-numbers",
            DEFAULT_REALM_NAME, target_user.id
        ),
        &token,
        serde_json::json!({
            "phone_number": "+1 555 0101",
            "is_primary": false,
            "is_verified": false
        }),
    );
    let add_second_res = ctx.request(add_second_req).await;
    assert_eq!(add_second_res.status(), StatusCode::CREATED);
    let second_json = json_body(add_second_res).await;
    let second_id = second_json["id"].as_str().expect("second id").to_string();

    let duplicate_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "duplicate-phone-user",
            "old-password-123",
            Some("duplicate-phone@example.com"),
            false,
        )
        .await
        .expect("create duplicate user");
    let duplicate_req = json_request(
        "POST",
        format!(
            "/api/realms/{}/users/{}/phone-numbers",
            DEFAULT_REALM_NAME, duplicate_user.id
        ),
        &token,
        serde_json::json!({ "phone_number": "+15550101" }),
    );
    let duplicate_res = ctx.request(duplicate_req).await;
    assert_eq!(duplicate_res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let duplicate_json = json_body(duplicate_res).await;
    assert_eq!(
        duplicate_json["fields"]["phone_number"],
        "Phone number is already in use"
    );

    let delete_primary_req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/users/{}/phone-numbers/{}",
            DEFAULT_REALM_NAME, target_user.id, primary_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("delete primary request");
    let delete_primary_res = ctx.request(delete_primary_req).await;
    assert_eq!(delete_primary_res.status(), StatusCode::CONFLICT);

    let promote_second_req = json_request(
        "PUT",
        format!(
            "/api/realms/{}/users/{}/phone-numbers/{}/primary",
            DEFAULT_REALM_NAME, target_user.id, second_id
        ),
        &token,
        serde_json::json!({}),
    );
    let promote_second_res = ctx.request(promote_second_req).await;
    assert_eq!(promote_second_res.status(), StatusCode::OK);

    let verify_second_req = json_request(
        "PATCH",
        format!(
            "/api/realms/{}/users/{}/phone-numbers/{}/verified",
            DEFAULT_REALM_NAME, target_user.id, second_id
        ),
        &token,
        serde_json::json!({ "is_verified": true }),
    );
    let verify_second_res = ctx.request(verify_second_req).await;
    assert_eq!(verify_second_res.status(), StatusCode::OK);

    let get_user_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/users/{}",
            DEFAULT_REALM_NAME, target_user.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("get user request");
    let get_user_res = ctx.request(get_user_req).await;
    assert_eq!(get_user_res.status(), StatusCode::OK);
    let get_user_json = json_body(get_user_res).await;
    assert_eq!(get_user_json["phone_number"], "+1 555 0101");
    assert_eq!(
        get_user_json["phone_numbers"]
            .as_array()
            .expect("phone number list")
            .len(),
        2
    );
    let primary_phone = get_user_json["phone_numbers"]
        .as_array()
        .expect("phone number list")
        .iter()
        .find(|value| value["id"] == second_id)
        .expect("second phone");
    assert_eq!(primary_phone["is_primary"], true);
    assert_eq!(primary_phone["is_verified"], true);
}
