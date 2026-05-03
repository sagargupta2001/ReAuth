use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature as P256Signature, SigningKey};
use p256::pkcs8::EncodePublicKey;
use serial_test::serial;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use reauth::application::flow_manager::UpdateDraftRequest;
use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_passkey_settings_service::UpdateRealmPasskeySettingsPayload;
use reauth::application::realm_service::CreateRealmPayload;
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::auth_session::AuthenticationSession;
use reauth::domain::permissions;

use crate::support::TestContext;

fn b64url(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(bytes)
}

fn deterministic_signing_key(seed: u8) -> SigningKey {
    let mut bytes = [seed; 32];
    if bytes[0] == 0 {
        bytes[0] = 1;
    }
    SigningKey::from_bytes((&bytes).into()).expect("deterministic signing key")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json body")
}

fn connect_info() -> ConnectInfo<std::net::SocketAddr> {
    ConnectInfo(std::net::SocketAddr::from((
        std::net::Ipv4Addr::new(127, 0, 0, 1),
        7001,
    )))
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

async fn setup_realm_writer_token(ctx: &TestContext, realm_id: Uuid) -> String {
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm_id,
            "realm-writer",
            "password",
            Some("writer@example.com"),
            false,
        )
        .await
        .expect("create writer user");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: "realm-writer".to_string(),
                description: Some("Realm writer".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm_id, role.id, permissions::REALM_READ.to_string())
        .await
        .expect("assign realm read");

    ctx.app_state
        .rbac_service
        .assign_permission_to_role(realm_id, role.id, permissions::REALM_WRITE.to_string())
        .await
        .expect("assign realm write");

    ctx.app_state
        .rbac_service
        .assign_role_to_user(realm_id, user.id, role.id)
        .await
        .expect("assign role to user");

    let (login, _) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");

    login.access_token
}

async fn enable_passkeys(ctx: &TestContext, realm_id: Uuid) {
    ctx.app_state
        .realm_passkey_settings_service
        .update_settings(
            realm_id,
            UpdateRealmPasskeySettingsPayload {
                enabled: Some(true),
                allow_password_fallback: Some(true),
                discoverable_preferred: Some(true),
                challenge_ttl_secs: Some(120),
                reauth_max_age_secs: Some(300),
            },
        )
        .await
        .expect("enable passkeys");
}

async fn publish_passkey_flow(
    ctx: &TestContext,
    realm: &reauth::domain::realm::Realm,
    node_id: &str,
    auth_type: &str,
    outputs: &[&str],
) -> Uuid {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let mut edges = vec![serde_json::json!({
        "id": "e-start-passkey",
        "source": "start",
        "target": node_id,
        "sourceHandle": "next"
    })];
    for output in outputs {
        edges.push(serde_json::json!({
            "id": format!("e-passkey-{}", output),
            "source": node_id,
            "target": "allow",
            "sourceHandle": output
        }));
    }

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": node_id, "type": auth_type, "data": { "config": { "auth_type": auth_type } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": edges
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
        .expect("update flow draft");
    ctx.app_state
        .flow_manager
        .publish_flow(realm.id, flow_id)
        .await
        .expect("publish flow");

    let version_num = ctx
        .app_state
        .flow_store
        .get_deployed_version_number(&realm.id, "browser", &flow_id)
        .await
        .expect("deployment lookup")
        .expect("deployed version number");
    let version = ctx
        .app_state
        .flow_store
        .get_version_by_number(&flow_id, version_num)
        .await
        .expect("version lookup")
        .expect("deployed version");
    Uuid::parse_str(&version.id).expect("parse version id")
}

async fn create_auth_session(
    ctx: &TestContext,
    realm_id: Uuid,
    flow_version_id: Uuid,
    current_node_id: &str,
    user_id: Option<Uuid>,
) -> Uuid {
    let mut session =
        AuthenticationSession::new(realm_id, flow_version_id, current_node_id.to_string());
    session.user_id = user_id;
    if let Some(user_id) = user_id {
        session.update_context("user_id", serde_json::json!(user_id.to_string()));
    }
    let id = session.id;
    ctx.app_state
        .auth_session_repo
        .create(&session)
        .await
        .expect("create auth session");
    id
}

fn origin_for_tests() -> String {
    "http://localhost:3000".to_string()
}

fn enrollment_auth_data(rp_id: &str, credential_id: &[u8]) -> Vec<u8> {
    let mut auth_data = Vec::new();
    let rp_hash = Sha256::digest(rp_id.as_bytes());
    auth_data.extend_from_slice(&rp_hash);
    auth_data.push(0x41); // UP + AT
    auth_data.extend_from_slice(&0u32.to_be_bytes());
    auth_data.extend_from_slice(&[0u8; 16]); // aaguid
    auth_data.extend_from_slice(&(credential_id.len() as u16).to_be_bytes());
    auth_data.extend_from_slice(credential_id);
    auth_data
}

fn assertion_auth_data(rp_id: &str, sign_count: u32) -> Vec<u8> {
    let mut auth_data = Vec::new();
    let rp_hash = Sha256::digest(rp_id.as_bytes());
    auth_data.extend_from_slice(&rp_hash);
    auth_data.push(0x01); // UP
    auth_data.extend_from_slice(&sign_count.to_be_bytes());
    auth_data
}

#[tokio::test]
#[serial(test_db)]
async fn passkey_enroll_options_and_verify_handler_work() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    enable_passkeys(&ctx, realm.id).await;

    let flow_version_id = publish_passkey_flow(
        &ctx,
        &realm,
        "passkey-enroll",
        "core.auth.passkey_enroll",
        &["success", "skip", "failure"],
    )
    .await;

    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "passkey-user",
            "password",
            Some("passkey@example.com"),
            false,
        )
        .await
        .expect("create user");
    let auth_session_id = create_auth_session(
        &ctx,
        realm.id,
        flow_version_id,
        "passkey-enroll",
        Some(user.id),
    )
    .await;

    let options_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/options",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "auth_session_id": auth_session_id }).to_string(),
        ))
        .expect("options request");
    let options_res = ctx.request(options_req).await;
    assert_eq!(options_res.status(), StatusCode::OK);
    let options_json = json_body(options_res).await;

    let challenge_id = options_json
        .get("challenge_id")
        .and_then(|value| value.as_str())
        .expect("challenge_id");
    let challenge = options_json
        .get("public_key")
        .and_then(|value| value.get("challenge"))
        .and_then(|value| value.as_str())
        .expect("public_key.challenge");
    let rp_id = options_json
        .get("public_key")
        .and_then(|value| value.get("rp"))
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .expect("public_key.rp.id");

    let credential_id = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let credential_id_b64 = b64url(&credential_id);
    let signing_key = deterministic_signing_key(7);
    let public_key_der = signing_key
        .verifying_key()
        .to_public_key_der()
        .expect("spki der")
        .as_bytes()
        .to_vec();
    let public_key_b64 = b64url(&public_key_der);

    let client_data_json = serde_json::json!({
        "type": "webauthn.create",
        "challenge": challenge,
        "origin": origin_for_tests()
    })
    .to_string();
    let verify_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/verify",
            DEFAULT_REALM_NAME
        ))
        .extension(connect_info())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "challenge_id": challenge_id,
                "friendly_name": "Laptop key",
                "credential": {
                    "id": credential_id_b64,
                    "type": "public-key",
                    "response": {
                        "clientDataJSON": b64url(client_data_json.as_bytes()),
                        "authenticatorData": b64url(&enrollment_auth_data(rp_id, &credential_id)),
                        "publicKey": public_key_b64,
                        "transports": ["internal"]
                    }
                }
            })
            .to_string(),
        ))
        .expect("verify request");
    let verify_res = ctx.request(verify_req).await;
    assert_eq!(verify_res.status(), StatusCode::OK);
    let verify_json = json_body(verify_res).await;
    assert_eq!(
        verify_json.get("status").and_then(|value| value.as_str()),
        Some("redirect")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn passkey_authenticate_verify_rejects_invalid_signature() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    enable_passkeys(&ctx, realm.id).await;

    let enroll_flow_version_id = publish_passkey_flow(
        &ctx,
        &realm,
        "passkey-enroll",
        "core.auth.passkey_enroll",
        &["success", "skip", "failure"],
    )
    .await;

    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "sig-user",
            "password",
            Some("sig@example.com"),
            false,
        )
        .await
        .expect("create user");

    let enroll_session_id = create_auth_session(
        &ctx,
        realm.id,
        enroll_flow_version_id,
        "passkey-enroll",
        Some(user.id),
    )
    .await;

    let enroll_options_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/options",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "auth_session_id": enroll_session_id }).to_string(),
        ))
        .expect("enroll options request");
    let enroll_options_res = ctx.request(enroll_options_req).await;
    assert_eq!(enroll_options_res.status(), StatusCode::OK);
    let enroll_options_json = json_body(enroll_options_res).await;
    let enroll_challenge_id = enroll_options_json
        .get("challenge_id")
        .and_then(|value| value.as_str())
        .expect("enroll challenge id");
    let enroll_challenge = enroll_options_json
        .get("public_key")
        .and_then(|value| value.get("challenge"))
        .and_then(|value| value.as_str())
        .expect("enroll challenge");
    let rp_id = enroll_options_json
        .get("public_key")
        .and_then(|value| value.get("rp"))
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .expect("rp id");

    let credential_id = vec![9u8, 8, 7, 6, 5, 4, 3, 2];
    let credential_id_b64 = b64url(&credential_id);
    let enrolled_signing_key = deterministic_signing_key(11);
    let enrolled_public_key_der = enrolled_signing_key
        .verifying_key()
        .to_public_key_der()
        .expect("spki der")
        .as_bytes()
        .to_vec();
    let enrolled_public_key_b64 = b64url(&enrolled_public_key_der);

    let enroll_client_data_json = serde_json::json!({
        "type": "webauthn.create",
        "challenge": enroll_challenge,
        "origin": origin_for_tests()
    })
    .to_string();
    let enroll_verify_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/verify",
            DEFAULT_REALM_NAME
        ))
        .extension(connect_info())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "challenge_id": enroll_challenge_id,
                "credential": {
                    "id": credential_id_b64,
                    "type": "public-key",
                    "response": {
                        "clientDataJSON": b64url(enroll_client_data_json.as_bytes()),
                        "authenticatorData": b64url(&enrollment_auth_data(rp_id, &credential_id)),
                        "publicKey": enrolled_public_key_b64
                    }
                }
            })
            .to_string(),
        ))
        .expect("enroll verify request");
    let enroll_verify_res = ctx.request(enroll_verify_req).await;
    assert_eq!(enroll_verify_res.status(), StatusCode::OK);

    let assert_flow_version_id = publish_passkey_flow(
        &ctx,
        &realm,
        "passkey-assert",
        "core.auth.passkey_assert",
        &["success", "fallback", "failure"],
    )
    .await;
    let assert_session_id = create_auth_session(
        &ctx,
        realm.id,
        assert_flow_version_id,
        "passkey-assert",
        None,
    )
    .await;

    let auth_options_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/authenticate/options",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "auth_session_id": assert_session_id,
                "identifier": "sig-user",
                "intent": "login"
            })
            .to_string(),
        ))
        .expect("auth options request");
    let auth_options_res = ctx.request(auth_options_req).await;
    assert_eq!(auth_options_res.status(), StatusCode::OK);
    let auth_options_json = json_body(auth_options_res).await;

    let challenge_id = auth_options_json
        .get("challenge_id")
        .and_then(|value| value.as_str())
        .expect("challenge id");
    let challenge = auth_options_json
        .get("public_key")
        .and_then(|value| value.get("challenge"))
        .and_then(|value| value.as_str())
        .expect("challenge");

    let client_data_json = serde_json::json!({
        "type": "webauthn.get",
        "challenge": challenge,
        "origin": origin_for_tests()
    })
    .to_string();
    let auth_data = assertion_auth_data(rp_id, 1);
    let client_data_hash = Sha256::digest(client_data_json.as_bytes());
    let mut signed_data = auth_data.clone();
    signed_data.extend_from_slice(&client_data_hash);

    let wrong_signing_key = deterministic_signing_key(17);
    let wrong_signature: P256Signature = wrong_signing_key.sign(&signed_data);
    let wrong_signature_der = wrong_signature.to_der().as_bytes().to_vec();

    let verify_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/authenticate/verify",
            DEFAULT_REALM_NAME
        ))
        .extension(connect_info())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "challenge_id": challenge_id,
                "credential": {
                    "id": credential_id_b64,
                    "type": "public-key",
                    "response": {
                        "clientDataJSON": b64url(client_data_json.as_bytes()),
                        "authenticatorData": b64url(&auth_data),
                        "signature": b64url(&wrong_signature_der)
                    }
                }
            })
            .to_string(),
        ))
        .expect("verify request");
    let verify_res = ctx.request(verify_req).await;
    assert_eq!(verify_res.status(), StatusCode::UNAUTHORIZED);
    let verify_json = json_body(verify_res).await;
    assert_eq!(
        verify_json.get("code").and_then(|value| value.as_str()),
        Some("auth.invalid_credentials")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn apply_recommended_passkey_browser_flow_enables_passkeys_and_publishes() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/passkey-settings/recommended-browser-flow",
            realm.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "enable_passkeys": true
            })
            .to_string(),
        ))
        .expect("request");
    let response = ctx.request(req).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = json_body(response).await;

    assert_eq!(
        json.get("settings")
            .and_then(|value| value.get("enabled"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    let version_id = json
        .get("browser_flow_version_id")
        .and_then(|value| value.as_str())
        .expect("version id");
    let _ = Uuid::parse_str(version_id).expect("version id uuid");

    let browser_flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("browser flow id");
    let deployed_number = ctx
        .app_state
        .flow_store
        .get_deployed_version_number(&realm.id, "browser", &browser_flow_id)
        .await
        .expect("deployed lookup")
        .expect("deployed version");
    let deployed = ctx
        .app_state
        .flow_store
        .get_version_by_number(&browser_flow_id, deployed_number)
        .await
        .expect("version lookup")
        .expect("deployed version row");
    assert!(deployed.graph_json.contains("core.auth.passkey_assert"));
}

#[tokio::test]
#[serial(test_db)]
async fn apply_recommended_passkey_registration_flow_publishes_enrollment_node() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/passkey-settings/recommended-registration-flow",
            realm.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("request");
    let response = ctx.request(req).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = json_body(response).await;

    assert_eq!(
        json.get("settings")
            .and_then(|value| value.get("enabled"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    let version_id = json
        .get("registration_flow_version_id")
        .and_then(|value| value.as_str())
        .expect("version id");
    let _ = Uuid::parse_str(version_id).expect("version id uuid");

    let registration_flow_id = realm
        .registration_flow_id
        .as_ref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("registration flow id");
    let deployed_number = ctx
        .app_state
        .flow_store
        .get_deployed_version_number(&realm.id, "registration", &registration_flow_id)
        .await
        .expect("deployed lookup")
        .expect("deployed version");
    let deployed = ctx
        .app_state
        .flow_store
        .get_version_by_number(&registration_flow_id, deployed_number)
        .await
        .expect("version lookup")
        .expect("deployed version row");
    assert!(deployed.graph_json.contains("core.auth.passkey_enroll"));
}

#[tokio::test]
#[serial(test_db)]
async fn get_passkey_analytics_reports_counts_and_failures() {
    let ctx = TestContext::new().await;
    let realm = setup_realm(&ctx).await;
    enable_passkeys(&ctx, realm.id).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let enroll_flow_version_id = publish_passkey_flow(
        &ctx,
        &realm,
        "passkey-enroll",
        "core.auth.passkey_enroll",
        &["success", "skip", "failure"],
    )
    .await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "analytics-user",
            "password",
            Some("analytics@example.com"),
            false,
        )
        .await
        .expect("create user");
    let enroll_session_id = create_auth_session(
        &ctx,
        realm.id,
        enroll_flow_version_id,
        "passkey-enroll",
        Some(user.id),
    )
    .await;

    let enroll_options_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/options",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "auth_session_id": enroll_session_id }).to_string(),
        ))
        .expect("enroll options");
    let enroll_options_res = ctx.request(enroll_options_req).await;
    assert_eq!(enroll_options_res.status(), StatusCode::OK);
    let enroll_options_json = json_body(enroll_options_res).await;

    let challenge_id = enroll_options_json
        .get("challenge_id")
        .and_then(|value| value.as_str())
        .expect("challenge id");
    let challenge = enroll_options_json
        .get("public_key")
        .and_then(|value| value.get("challenge"))
        .and_then(|value| value.as_str())
        .expect("challenge");
    let rp_id = enroll_options_json
        .get("public_key")
        .and_then(|value| value.get("rp"))
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .expect("rp id");

    let credential_id = vec![3u8, 1, 4, 1, 5, 9, 2, 6];
    let credential_id_b64 = b64url(&credential_id);
    let signing_key = deterministic_signing_key(23);
    let public_key_der = signing_key
        .verifying_key()
        .to_public_key_der()
        .expect("spki der")
        .as_bytes()
        .to_vec();
    let client_data_json = serde_json::json!({
        "type": "webauthn.create",
        "challenge": challenge,
        "origin": origin_for_tests()
    })
    .to_string();

    let enroll_verify_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/enroll/verify",
            DEFAULT_REALM_NAME
        ))
        .extension(connect_info())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "challenge_id": challenge_id,
                "credential": {
                    "id": credential_id_b64,
                    "type": "public-key",
                    "response": {
                        "clientDataJSON": b64url(client_data_json.as_bytes()),
                        "authenticatorData": b64url(&enrollment_auth_data(rp_id, &credential_id)),
                        "publicKey": b64url(&public_key_der)
                    }
                }
            })
            .to_string(),
        ))
        .expect("enroll verify");
    let enroll_verify_res = ctx.request(enroll_verify_req).await;
    assert_eq!(enroll_verify_res.status(), StatusCode::OK);

    // Seed a recent suspicious event for diagnostics.
    ctx.app_state
        .audit_service
        .record(reauth::domain::audit::NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: Some(user.id),
            action: "passkey.assertion.invalid_signature".to_string(),
            target_type: "passkey".to_string(),
            target_id: Some(credential_id_b64.clone()),
            metadata: serde_json::json!({ "source": "integration-test" }),
        })
        .await
        .expect("record audit");

    let assert_flow_version_id = publish_passkey_flow(
        &ctx,
        &realm,
        "passkey-assert",
        "core.auth.passkey_assert",
        &["success", "fallback", "failure"],
    )
    .await;
    let assert_session_id = create_auth_session(
        &ctx,
        realm.id,
        assert_flow_version_id,
        "passkey-assert",
        None,
    )
    .await;

    let options_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/passkeys/authenticate/options",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "auth_session_id": assert_session_id,
                "identifier": "analytics-user",
                "intent": "login"
            })
            .to_string(),
        ))
        .expect("assert options");
    let options_res = ctx.request(options_req).await;
    assert_eq!(options_res.status(), StatusCode::OK);

    let analytics_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/passkey-settings/analytics?window_hours={}&recent_limit={}",
            realm.id, 24, 5
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("analytics request");
    let analytics_res = ctx.request(analytics_req).await;
    assert_eq!(analytics_res.status(), StatusCode::OK);
    let analytics_json = json_body(analytics_res).await;

    assert_eq!(
        analytics_json
            .get("credentials_total")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert!(
        analytics_json
            .get("challenges")
            .and_then(|value| value.get("pending_total"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 1
    );
    assert!(analytics_json
        .get("challenges")
        .and_then(|value| value.get("pending_expired"))
        .and_then(|value| value.as_u64())
        .is_some());
    assert_eq!(
        analytics_json
            .get("outcomes")
            .and_then(|value| value.get("assertion_invalid_signature"))
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert!(analytics_json
        .get("recent_failures")
        .and_then(|value| value.as_array())
        .map(|value| !value.is_empty())
        .unwrap_or(false));
}
