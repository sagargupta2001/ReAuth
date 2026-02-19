use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, HeaderMap, Request, StatusCode};
use http_body_util::BodyExt;
use serial_test::serial;
use std::net::{Ipv4Addr, SocketAddr};
use uuid::Uuid;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

use reauth_core::application::flow_manager::UpdateDraftRequest;
use reauth_core::application::realm_service::CreateRealmPayload;
use reauth_core::constants::{DEFAULT_REALM_NAME, LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE};
use reauth_core::domain::auth_session::{AuthenticationSession, SessionStatus};
use reauth_core::domain::oidc::OidcClient;
use reauth_core::domain::realm::Realm;

use crate::support::TestContext;

fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    for value in headers.get_all(header::SET_COOKIE).iter() {
        let set_cookie = match value.to_str() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let first = match set_cookie.split(';').next() {
            Some(v) => v,
            None => continue,
        };
        let mut parts = first.splitn(2, '=');
        let key = parts.next().map(str::trim);
        let val = parts.next().map(str::trim);
        if key == Some(name) {
            if let Some(val) = val {
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

fn cookie_present(headers: &HeaderMap, name: &str) -> bool {
    headers.get_all(header::SET_COOKIE).iter().any(|value| {
        let set_cookie = match value.to_str() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let first = match set_cookie.split(';').next() {
            Some(v) => v,
            None => return false,
        };
        let mut parts = first.splitn(2, '=');
        let key = parts.next().map(str::trim);
        key == Some(name)
    })
}

async fn assert_error_response(response: axum::response::Response, status: StatusCode) {
    assert_eq!(response.status(), status);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("error json");
    let message = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!message.is_empty());
}

async fn setup_master_realm(ctx: &TestContext) -> Realm {
    ctx.app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        })
        .await
        .expect("create realm")
}

async fn ensure_minimal_browser_flow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-allow", "source": "start", "target": "allow", "sourceHandle": "next" }
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

    ctx.app_state
        .flow_manager
        .publish_flow(realm.id, flow_id)
        .await
        .expect("publish flow");
}

async fn ensure_password_browser_flow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "auth-password", "type": "core.auth.password", "data": { "config": { "auth_type": "core.auth.password" } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-password", "source": "start", "target": "auth-password", "sourceHandle": "next" },
            { "id": "e-password-allow", "source": "auth-password", "target": "allow", "sourceHandle": "success" }
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

    ctx.app_state
        .flow_manager
        .publish_flow(realm.id, flow_id)
        .await
        .expect("publish flow");
}

async fn register_oidc_client(
    ctx: &TestContext,
    realm_id: Uuid,
    client_id: &str,
    redirect_uri: &str,
) -> OidcClient {
    let mut client = OidcClient {
        id: Uuid::new_v4(),
        realm_id,
        client_id: client_id.to_string(),
        client_secret: None,
        redirect_uris: serde_json::to_string(&vec![redirect_uri.to_string()])
            .expect("redirect_uris json"),
        scopes: serde_json::to_string(&vec!["openid"]).expect("scopes json"),
        web_origins: serde_json::to_string(&Vec::<String>::new()).expect("web_origins json"),
        managed_by_config: false,
    };

    ctx.app_state
        .oidc_service
        .register_client(&mut client)
        .await
        .expect("register client");

    client
}

async fn active_browser_flow_version_id(ctx: &TestContext, realm: &Realm) -> Uuid {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let version = ctx
        .app_state
        .flow_store
        .get_active_version(&flow_id)
        .await
        .expect("active version")
        .expect("active version missing");

    Uuid::parse_str(&version.id).expect("version id")
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_authorize_sets_login_cookie_and_redirects() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_minimal_browser_flow(&ctx, &realm).await;

    let redirect_uri = "http://localhost/callback";
    let client_id = "client-app";
    let _client = register_oidc_client(&ctx, realm.id, client_id, redirect_uri).await;

    let code_verifier = "verifier123";
    let code_challenge = pkce_challenge(code_verifier);

    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid")
        .append_pair("state", "abc123")
        .append_pair("nonce", "nonce-1")
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .finish();

    let uri = format!(
        "/api/realms/{}/oidc/authorize?{}",
        DEFAULT_REALM_NAME, query
    );
    let response = ctx
        .request(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await;

    assert!(response.status().is_redirection());

    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(location.starts_with("/#/login?realm=master"));
    assert!(location.contains("client_id=client-app"));

    let session_id = cookie_value(response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");

    let session = ctx
        .app_state
        .auth_session_repo
        .find_by_id(&session_id)
        .await
        .expect("session lookup")
        .expect("session missing");

    assert_eq!(session.realm_id, realm.id);
    assert_eq!(
        session
            .context
            .get("oidc")
            .and_then(|v| v.get("client_id"))
            .and_then(|v| v.as_str()),
        Some(client_id)
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_exchange_returns_tokens_and_refresh_cookie() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let redirect_uri = "http://localhost/callback";
    let client_id = "client-app";
    let _client = register_oidc_client(&ctx, realm.id, client_id, redirect_uri).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "alice", "password-123")
        .await
        .expect("create user");

    let code_verifier = "verifier-token";
    let code_challenge = pkce_challenge(code_verifier);

    let auth_code = ctx
        .app_state
        .oidc_service
        .create_authorization_code(
            realm.id,
            user.id,
            client_id.to_string(),
            redirect_uri.to_string(),
            None,
            Some(code_challenge),
            "S256".to_string(),
        )
        .await
        .expect("create auth code");

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", &auth_code.code)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("client_id", client_id)
        .append_pair("code_verifier", code_verifier)
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let refresh_id = cookie_value(response.headers(), REFRESH_TOKEN_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("refresh token cookie");

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).expect("token json");

    assert!(
        json.get("access_token")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .len()
            > 10
    );
    assert!(
        json.get("id_token")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .len()
            > 10
    );
    assert_eq!(
        json.get("token_type").and_then(|v| v.as_str()),
        Some("Bearer")
    );

    let refresh_token = ctx
        .app_state
        .session_repo
        .find_by_id(&refresh_id)
        .await
        .expect("refresh token lookup")
        .expect("refresh token missing");

    assert_eq!(refresh_token.user_id, user.id);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_refresh_rotates_refresh_token() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "bob", "password-123")
        .await
        .expect("create user");

    let (_login, refresh_token) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");

    let cookie_header = format!("{}={}", REFRESH_TOKEN_COOKIE, refresh_token.id);
    let response = ctx
        .request(
            Request::builder()
                .method("POST")
                .uri(format!("/api/realms/{}/auth/refresh", DEFAULT_REALM_NAME))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let new_refresh_id = cookie_value(response.headers(), REFRESH_TOKEN_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("new refresh token cookie");
    assert_ne!(new_refresh_id, refresh_token.id);

    let old = ctx
        .app_state
        .session_repo
        .find_by_id(&refresh_token.id)
        .await
        .expect("old refresh lookup");
    assert!(old.is_none());

    let fresh = ctx
        .app_state
        .session_repo
        .find_by_id(&new_refresh_id)
        .await
        .expect("new refresh lookup");
    assert!(fresh.is_some());
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_flow_challenge_and_execute_success() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "charlie", "password-123")
        .await
        .expect("create user");

    let mut start_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    start_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let start_response = ctx.request(start_request).await;
    assert_eq!(start_response.status(), StatusCode::OK);

    let session_id = cookie_value(start_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");

    let body_bytes = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).expect("challenge json");

    assert_eq!(
        json.get("status").and_then(|v| v.as_str()),
        Some("challenge")
    );
    assert_eq!(
        json.get("challengeName").and_then(|v| v.as_str()),
        Some("login-password")
    );

    let payload = serde_json::json!({
        "username": user.username,
        "password": "password-123"
    });

    let mut exec_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/login/execute",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::from(payload.to_string()))
        .unwrap();
    exec_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let exec_response = ctx.request(exec_request).await;
    assert_eq!(exec_response.status(), StatusCode::OK);

    let refresh_id = cookie_value(exec_response.headers(), REFRESH_TOKEN_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("refresh token cookie");

    let exec_body = exec_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let exec_json: serde_json::Value = serde_json::from_slice(&exec_body).expect("redirect json");
    assert_eq!(
        exec_json.get("status").and_then(|v| v.as_str()),
        Some("redirect")
    );
    assert_eq!(exec_json.get("url").and_then(|v| v.as_str()), Some("/"));

    let refresh_token = ctx
        .app_state
        .session_repo
        .find_by_id(&refresh_id)
        .await
        .expect("refresh token lookup")
        .expect("refresh token missing");
    assert_eq!(refresh_token.user_id, user.id);
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_authorize_rejects_invalid_redirect_uri() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let redirect_uri = "http://localhost/callback";
    let client_id = "client-app";
    let _client = register_oidc_client(&ctx, realm.id, client_id, redirect_uri).await;

    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", "http://evil.invalid/callback")
        .append_pair("response_type", "code")
        .finish();

    let uri = format!(
        "/api/realms/{}/oidc/authorize?{}",
        DEFAULT_REALM_NAME, query
    );
    let response = ctx
        .request(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(cookie_value(response.headers(), LOGIN_SESSION_COOKIE).is_none());
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_authorize_rejects_missing_client_id() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("redirect_uri", "http://localhost/callback")
        .append_pair("response_type", "code")
        .finish();

    let uri = format!(
        "/api/realms/{}/oidc/authorize?{}",
        DEFAULT_REALM_NAME, query
    );
    let response = ctx
        .request(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_authorize_rejects_missing_redirect_uri() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", "client-app")
        .append_pair("response_type", "code")
        .finish();

    let uri = format!(
        "/api/realms/{}/oidc/authorize?{}",
        DEFAULT_REALM_NAME, query
    );
    let response = ctx
        .request(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_authorize_rejects_missing_response_type() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", "client-app")
        .append_pair("redirect_uri", "http://localhost/callback")
        .finish();

    let uri = format!(
        "/api/realms/{}/oidc/authorize?{}",
        DEFAULT_REALM_NAME, query
    );
    let response = ctx
        .request(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_exchange_rejects_invalid_pkce_verifier() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let redirect_uri = "http://localhost/callback";
    let client_id = "client-app";
    let _client = register_oidc_client(&ctx, realm.id, client_id, redirect_uri).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "dana", "password-123")
        .await
        .expect("create user");

    let code_verifier = "verifier-good";
    let code_challenge = pkce_challenge(code_verifier);

    let auth_code = ctx
        .app_state
        .oidc_service
        .create_authorization_code(
            realm.id,
            user.id,
            client_id.to_string(),
            redirect_uri.to_string(),
            None,
            Some(code_challenge),
            "S256".to_string(),
        )
        .await
        .expect("create auth code");

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", &auth_code.code)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("client_id", client_id)
        .append_pair("code_verifier", "wrong-verifier")
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_refresh_rejects_invalid_token_cookie() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let response = ctx
        .request(
            Request::builder()
                .method("POST")
                .uri(format!("/api/realms/{}/auth/refresh", DEFAULT_REALM_NAME))
                .header(
                    header::COOKIE,
                    format!("{}=not-a-uuid", REFRESH_TOKEN_COOKIE),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_execute_rejects_invalid_password_with_challenge() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "erin", "password-123")
        .await
        .expect("create user");

    let mut start_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    start_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let start_response = ctx.request(start_request).await;
    assert_eq!(start_response.status(), StatusCode::OK);

    let session_id = cookie_value(start_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");

    let payload = serde_json::json!({
        "username": user.username,
        "password": "wrong-password"
    });

    let mut exec_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/auth/login/execute",
            DEFAULT_REALM_NAME
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::from(payload.to_string()))
        .unwrap();
    exec_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let exec_response = ctx.request(exec_request).await;
    assert_eq!(exec_response.status(), StatusCode::OK);

    let body = exec_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        json.get("status").and_then(|v| v.as_str()),
        Some("challenge")
    );
    assert_eq!(
        json.get("challengeName").and_then(|v| v.as_str()),
        Some("login-password")
    );
    assert_eq!(
        json.get("context")
            .and_then(|ctx| ctx.get("error"))
            .and_then(|v| v.as_str()),
        Some("Invalid credentials")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_execute_requires_session_cookie() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let response = ctx
        .request({
            let mut request = Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/realms/{}/auth/login/execute",
                    DEFAULT_REALM_NAME
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "username": "someone",
                        "password": "password"
                    })
                    .to_string(),
                ))
                .unwrap();
            request
                .extensions_mut()
                .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
            request
        })
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_execute_rejects_session_from_other_realm() {
    let ctx = TestContext::new().await;
    let master = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &master).await;

    let _other = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "other".to_string(),
        })
        .await
        .expect("create realm");

    let mut start_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    start_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let start_response = ctx.request(start_request).await;
    assert_eq!(start_response.status(), StatusCode::OK);

    let session_id = cookie_value(start_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");

    let mut exec_request = Request::builder()
        .method("POST")
        .uri("/api/realms/other/auth/login/execute")
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::from(
            serde_json::json!({
                "username": "someone",
                "password": "password"
            })
            .to_string(),
        ))
        .unwrap();
    exec_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let exec_response = ctx.request(exec_request).await;
    assert_eq!(exec_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_prompt_login_forces_new_session() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    let version_id = active_browser_flow_version_id(&ctx, &realm).await;
    let existing_id = Uuid::new_v4();
    let session = AuthenticationSession {
        id: existing_id,
        realm_id: realm.id,
        flow_version_id: version_id,
        current_node_id: "auth-password".to_string(),
        context: serde_json::json!({}),
        status: SessionStatus::Active,
        user_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
    };

    ctx.app_state
        .auth_session_repo
        .create(&session)
        .await
        .expect("create session");

    let mut request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/auth/login?prompt=login",
            DEFAULT_REALM_NAME
        ))
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, existing_id),
        )
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let new_session_id = cookie_value(response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");

    assert_ne!(new_session_id, existing_id);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_resume_injects_oidc_and_sso_context() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    let version_id = active_browser_flow_version_id(&ctx, &realm).await;
    let existing_id = Uuid::new_v4();
    let session = AuthenticationSession {
        id: existing_id,
        realm_id: realm.id,
        flow_version_id: version_id,
        current_node_id: "auth-password".to_string(),
        context: serde_json::json!({}),
        status: SessionStatus::Active,
        user_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
    };

    ctx.app_state
        .auth_session_repo
        .create(&session)
        .await
        .expect("create session");

    let refresh_id = Uuid::new_v4();
    let code_verifier = "resume-verifier";
    let code_challenge = pkce_challenge(code_verifier);

    let uri = format!(
        "/api/realms/{}/auth/login?client_id=client-app&redirect_uri=http://localhost/callback&response_type=code&scope=openid&state=state1&nonce=nonce1&code_challenge={}&code_challenge_method=S256",
        DEFAULT_REALM_NAME,
        code_challenge
    );

    let mut request = Request::builder()
        .method("GET")
        .uri(uri)
        .header(
            header::COOKIE,
            format!(
                "{}={}; {}={}",
                LOGIN_SESSION_COOKIE, existing_id, REFRESH_TOKEN_COOKIE, refresh_id
            ),
        )
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let resumed_id = cookie_value(response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|val| Uuid::parse_str(&val).ok())
        .expect("login session cookie");
    assert_eq!(resumed_id, existing_id);

    let updated = ctx
        .app_state
        .auth_session_repo
        .find_by_id(&existing_id)
        .await
        .expect("session lookup")
        .expect("session missing");

    let refresh_str = refresh_id.to_string();
    assert_eq!(
        updated.context.get("sso_token_id").and_then(|v| v.as_str()),
        Some(refresh_str.as_str())
    );
    assert_eq!(
        updated
            .context
            .get("oidc")
            .and_then(|v| v.get("client_id"))
            .and_then(|v| v.as_str()),
        Some("client-app")
    );
    assert_eq!(
        updated
            .context
            .get("oidc")
            .and_then(|v| v.get("code_challenge"))
            .and_then(|v| v.as_str()),
        Some(code_challenge.as_str())
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_rejects_invalid_grant_type() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "password")
        .append_pair("code", "code123")
        .append_pair("redirect_uri", "http://localhost/callback")
        .append_pair("client_id", "client-app")
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_error_response(response, StatusCode::BAD_REQUEST).await;
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_rejects_missing_code() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("redirect_uri", "http://localhost/callback")
        .append_pair("client_id", "client-app")
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_error_response(response, StatusCode::UNPROCESSABLE_ENTITY).await;
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_rejects_missing_client_id() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", "code123")
        .append_pair("redirect_uri", "http://localhost/callback")
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_error_response(response, StatusCode::UNPROCESSABLE_ENTITY).await;
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_rejects_invalid_code() {
    let ctx = TestContext::new().await;
    let _realm = setup_master_realm(&ctx).await;

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", "invalid-code")
        .append_pair("redirect_uri", "http://localhost/callback")
        .append_pair("client_id", "client-app")
        .append_pair("code_verifier", "verifier")
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_error_response(response, StatusCode::UNAUTHORIZED).await;
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_token_rejects_invalid_redirect_uri() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let redirect_uri = "http://localhost/callback";
    let client_id = "client-app";
    let _client = register_oidc_client(&ctx, realm.id, client_id, redirect_uri).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "gina", "password-123")
        .await
        .expect("create user");

    let code_verifier = "verifier-redirect";
    let code_challenge = pkce_challenge(code_verifier);

    let auth_code = ctx
        .app_state
        .oidc_service
        .create_authorization_code(
            realm.id,
            user.id,
            client_id.to_string(),
            redirect_uri.to_string(),
            None,
            Some(code_challenge),
            "S256".to_string(),
        )
        .await
        .expect("create auth code");

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", &auth_code.code)
        .append_pair("redirect_uri", "http://evil.invalid/callback")
        .append_pair("client_id", client_id)
        .append_pair("code_verifier", code_verifier)
        .finish();

    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/realms/{}/oidc/token", DEFAULT_REALM_NAME))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_error_response(response, StatusCode::BAD_REQUEST).await;
}

#[tokio::test]
#[serial(test_db)]
async fn auth_logout_clears_cookies_and_revokes_refresh_token() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "frank", "password-123")
        .await
        .expect("create user");

    let (_login, refresh_token) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");

    let response = ctx
        .request(
            Request::builder()
                .method("POST")
                .uri(format!("/api/realms/{}/auth/logout", DEFAULT_REALM_NAME))
                .header(
                    header::COOKIE,
                    format!("{}={}", REFRESH_TOKEN_COOKIE, refresh_token.id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(cookie_present(response.headers(), REFRESH_TOKEN_COOKIE));
    assert!(cookie_present(response.headers(), LOGIN_SESSION_COOKIE));

    let removed = ctx
        .app_state
        .session_repo
        .find_by_id(&refresh_token.id)
        .await
        .expect("session lookup");
    assert!(removed.is_none());
}
