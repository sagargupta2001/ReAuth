use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use http_body_util::BodyExt;
use serial_test::serial;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use url::Url;
use uuid::Uuid;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use sha2::{Digest, Sha256};

use reauth::application::flow_manager::{CreateDraftRequest, UpdateDraftRequest};
use reauth::application::idp_service::{
    CreateIdentityProviderRequest, IdentityProviderResponse, UpdateIdentityProviderRequest,
};
use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_idp_settings_service::UpdateRealmIdpSettingsPayload;
use reauth::application::realm_service::{CreateRealmPayload, UpdateRealmPayload};
use reauth::bootstrap::app_state::SetupState;
use reauth::constants::{DEFAULT_REALM_NAME, LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE};
use reauth::domain::audit::NewAuditEvent;
use reauth::domain::auth_session::{AuthenticationSession, SessionStatus};
use reauth::domain::identity_provider::{IdentityProviderProtocol, OAuthBrokerResult};
use reauth::domain::oidc::OidcClient;
use reauth::domain::permissions;
use reauth::domain::realm::Realm;
use reauth::domain::user::User;

use crate::support::TestContext;

#[derive(Clone)]
struct FakeOauthScenario {
    userinfo_status: u16,
    userinfo_body: serde_json::Value,
    id_token: Option<String>,
    emails_status: u16,
    emails_body: serde_json::Value,
}

#[derive(Clone, Default)]
struct FakeOauthUpstreamState {
    scenarios: Arc<RwLock<HashMap<String, FakeOauthScenario>>>,
    jwks_body: Arc<serde_json::Value>,
}

struct FakeOauthUpstream {
    base_url: String,
    state: FakeOauthUpstreamState,
    task: JoinHandle<()>,
}

impl FakeOauthUpstream {
    async fn start() -> Self {
        let state = FakeOauthUpstreamState {
            scenarios: Arc::new(RwLock::new(HashMap::new())),
            jwks_body: Arc::new(oidc_test_jwks()),
        };
        let app = Router::new()
            .route(
                "/.well-known/openid-configuration",
                get(fake_discovery_handler),
            )
            .route("/authorize", get(fake_authorize_handler))
            .route("/token", post(fake_token_handler))
            .route("/userinfo", get(fake_userinfo_handler))
            .route("/user", get(fake_github_user_handler))
            .route("/user/emails", get(fake_user_emails_handler))
            .route("/jwks", get(fake_jwks_handler))
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake oauth upstream");
        let address = listener.local_addr().expect("fake upstream addr");
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve fake oauth upstream");
        });

        Self {
            base_url: format!("http://{}", address),
            state,
            task,
        }
    }

    async fn set_userinfo(&self, code: &str, userinfo_body: serde_json::Value) {
        self.set_scenario(
            code,
            StatusCode::OK,
            userinfo_body,
            None,
            StatusCode::NOT_FOUND,
            serde_json::json!({ "error": "missing_emails" }),
        )
        .await;
    }

    async fn set_oidc_userinfo(
        &self,
        code: &str,
        userinfo_body: serde_json::Value,
        id_token: String,
    ) {
        self.set_scenario(
            code,
            StatusCode::OK,
            userinfo_body,
            Some(id_token),
            StatusCode::NOT_FOUND,
            serde_json::json!({ "error": "missing_emails" }),
        )
        .await;
    }

    async fn set_github_profile(
        &self,
        code: &str,
        userinfo_body: serde_json::Value,
        emails_body: serde_json::Value,
    ) {
        self.set_scenario(
            code,
            StatusCode::OK,
            userinfo_body,
            None,
            StatusCode::OK,
            emails_body,
        )
        .await;
    }

    async fn set_scenario(
        &self,
        code: &str,
        status: StatusCode,
        userinfo_body: serde_json::Value,
        id_token: Option<String>,
        emails_status: StatusCode,
        emails_body: serde_json::Value,
    ) {
        self.state.scenarios.write().await.insert(
            code.to_string(),
            FakeOauthScenario {
                userinfo_status: status.as_u16(),
                userinfo_body,
                id_token,
                emails_status: emails_status.as_u16(),
                emails_body,
            },
        );
    }
}

impl Drop for FakeOauthUpstream {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn fake_authorize_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn fake_discovery_handler(headers: HeaderMap) -> impl IntoResponse {
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1");
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "authorization_endpoint": format!("http://{host}/authorize"),
            "token_endpoint": format!("http://{host}/token"),
            "userinfo_endpoint": format!("http://{host}/userinfo"),
            "jwks_uri": format!("http://{host}/jwks")
        })),
    )
        .into_response()
}

async fn fake_token_handler(
    State(state): State<FakeOauthUpstreamState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    let form = url::form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let Some(code) = form.get("code").cloned() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "missing_code" })),
        )
            .into_response();
    };
    if !state.scenarios.read().await.contains_key(&code) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "unknown_code" })),
        )
            .into_response();
    }

    let token_response = serde_json::json!({
        "access_token": format!("token-{}", code),
        "id_token": state
            .scenarios
            .read()
            .await
            .get(&code)
            .and_then(|scenario| scenario.id_token.clone())
    });

    let accepts_json = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("application/json"))
        .unwrap_or(false);
    if accepts_json {
        return (StatusCode::OK, Json(token_response)).into_response();
    }

    let access_token = token_response
        .get("access_token")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let mut encoded = url::form_urlencoded::Serializer::new(String::new());
    encoded.append_pair("access_token", access_token);
    encoded.append_pair("token_type", "bearer");
    if let Some(id_token) = token_response
        .get("id_token")
        .and_then(|value| value.as_str())
    {
        encoded.append_pair("id_token", id_token);
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-www-form-urlencoded")],
        encoded.finish(),
    )
        .into_response()
}

async fn fake_userinfo_handler(
    State(state): State<FakeOauthUpstreamState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let code = extract_code_from_bearer(&headers);
    let Some(code) = code else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "missing_token" })),
        )
            .into_response();
    };

    let Some(scenario) = state.scenarios.read().await.get(&code).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "unknown_code" })),
        )
            .into_response();
    };

    (
        StatusCode::from_u16(scenario.userinfo_status).expect("userinfo status"),
        Json(scenario.userinfo_body),
    )
        .into_response()
}

async fn fake_github_user_handler(
    State(state): State<FakeOauthUpstreamState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let has_user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let accepts_github_json = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("application/vnd.github+json"))
        .unwrap_or(false);
    if !has_user_agent || !accepts_github_json {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "message": "GitHub API requires User-Agent and vendor Accept headers"
            })),
        )
            .into_response();
    }

    let Some(code) = extract_code_from_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "missing_token" })),
        )
            .into_response();
    };
    let Some(scenario) = state.scenarios.read().await.get(&code).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "unknown_code" })),
        )
            .into_response();
    };

    (
        StatusCode::from_u16(scenario.userinfo_status).expect("userinfo status"),
        Json(scenario.userinfo_body),
    )
        .into_response()
}

async fn fake_user_emails_handler(
    State(state): State<FakeOauthUpstreamState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(code) = extract_code_from_bearer(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "missing_token" })),
        )
            .into_response();
    };
    let Some(scenario) = state.scenarios.read().await.get(&code).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "unknown_code" })),
        )
            .into_response();
    };

    (
        StatusCode::from_u16(scenario.emails_status).expect("emails status"),
        Json(scenario.emails_body),
    )
        .into_response()
}

async fn fake_jwks_handler(State(state): State<FakeOauthUpstreamState>) -> impl IntoResponse {
    (StatusCode::OK, Json((*state.jwks_body).clone())).into_response()
}

const TEST_OIDC_KEY_ID: &str = "rsa01";
const TEST_OIDC_RSA_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDJETqse41HRBsc
7cfcq3ak4oZWFCoZlcic525A3FfO4qW9BMtRO/iXiyCCHn8JhiL9y8j5JdVP2Q9Z
IpfElcFd3/guS9w+5RqQGgCR+H56IVUyHZWtTJbKPcwWXQdNUX0rBFcsBzCRESJL
eelOEdHIjG7LRkx5l/FUvlqsyHDVJEQsHwegZ8b8C0fz0EgT2MMEdn10t6Ur1rXz
jMB/wvCg8vG8lvciXmedyo9xJ8oMOh0wUEgxziVDMMovmC+aJctcHUAYubwoGN8T
yzcvnGqL7JSh36Pwy28iPzXZ2RLhAyJFU39vLaHdljwthUaupldlNyCfa6Ofy4qN
ctlUPlN1AgMBAAECggEAdESTQjQ70O8QIp1ZSkCYXeZjuhj081CK7jhhp/4ChK7J
GlFQZMwiBze7d6K84TwAtfQGZhQ7km25E1kOm+3hIDCoKdVSKch/oL54f/BK6sKl
qlIzQEAenho4DuKCm3I4yAw9gEc0DV70DuMTR0LEpYyXcNJY3KNBOTjN5EYQAR9s
2MeurpgK2MdJlIuZaIbzSGd+diiz2E6vkmcufJLtmYUT/k/ddWvEtz+1DnO6bRHh
xuuDMeJA/lGB/EYloSLtdyCF6sII6C6slJJtgfb0bPy7l8VtL5iDyz46IKyzdyzW
tKAn394dm7MYR1RlUBEfqFUyNK7C+pVMVoTwCC2V4QKBgQD64syfiQ2oeUlLYDm4
CcKSP3RnES02bcTyEDFSuGyyS1jldI4A8GXHJ/lG5EYgiYa1RUivge4lJrlNfjyf
dV230xgKms7+JiXqag1FI+3mqjAgg4mYiNjaao8N8O3/PD59wMPeWYImsWXNyeHS
55rUKiHERtCcvdzKl4u35ZtTqQKBgQDNKnX2bVqOJ4WSqCgHRhOm386ugPHfy+8j
m6cicmUR46ND6ggBB03bCnEG9OtGisxTo/TuYVRu3WP4KjoJs2LD5fwdwJqpgtHl
yVsk45Y1Hfo+7M6lAuR8rzCi6kHHNb0HyBmZjysHWZsn79ZM+sQnLpgaYgQGRbKV
DZWlbw7g7QKBgQCl1u+98UGXAP1jFutwbPsx40IVszP4y5ypCe0gqgon3UiY/G+1
zTLp79GGe/SjI2VpQ7AlW7TI2A0bXXvDSDi3/5Dfya9ULnFXv9yfvH1QwWToySpW
Kvd1gYSoiX84/WCtjZOr0e0HmLIb0vw0hqZA4szJSqoxQgvF22EfIWaIaQKBgQCf
34+OmMYw8fEvSCPxDxVvOwW2i7pvV14hFEDYIeZKW2W1HWBhVMzBfFB5SE8yaCQy
pRfOzj9aKOCm2FjjiErVNpkQoi6jGtLvScnhZAt/lr2TXTrl8OwVkPrIaN0bG/AS
aUYxmBPCpXu3UjhfQiWqFq/mFyzlqlgvuCc9g95HPQKBgAscKP8mLxdKwOgX8yFW
GcZ0izY/30012ajdHY+/QK5lsMoxTnn0skdS+spLxaS5ZEO4qvPVb8RAoCkWMMal
2pOhmquJQVDPDLuZHdrIiKiDM20dy9sMfHygWcZjQ4WSxf/J7T9canLZIXFhHAZT
3wc9h4G8BBCtWN2TN/LsGZdB
-----END PRIVATE KEY-----";

fn oidc_test_jwks() -> serde_json::Value {
    serde_json::json!({
        "keys": [
            {
                "kty": "RSA",
                "n": "yRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5_CYYi_cvI-SXVT9kPWSKXxJXBXd_4LkvcPuUakBoAkfh-eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG_AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi-yUod-j8MtvIj812dkS4QMiRVN_by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQ",
                "e": "AQAB",
                "kid": TEST_OIDC_KEY_ID,
                "alg": "RS256",
                "use": "sig"
            }
        ]
    })
}

fn issue_test_id_token(
    issuer: &str,
    audience: &str,
    subject: &str,
    nonce: &str,
    extra_claims: serde_json::Value,
) -> String {
    let mut claims = serde_json::json!({
        "iss": issuer,
        "aud": audience,
        "sub": subject,
        "nonce": nonce,
        "exp": Utc::now().timestamp() + 600,
    });
    let claim_map = claims
        .as_object_mut()
        .expect("oidc id token claims should be an object");
    for (key, value) in extra_claims
        .as_object()
        .expect("oidc extra claims should be an object")
    {
        claim_map.insert(key.clone(), value.clone());
    }

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_OIDC_KEY_ID.to_string());

    encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_OIDC_RSA_PRIVATE_KEY.as_bytes())
            .expect("oidc test private key"),
    )
    .expect("issue oidc id token")
}

fn extract_code_from_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer token-"))
        .map(str::to_string)
}

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
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        })
        .await
        .expect("create realm");

    let mut setup_state = ctx.app_state.setup_state.write().await;
    *setup_state = SetupState::sealed();

    realm
}

async fn setup_realm_writer_token(ctx: &TestContext, realm_id: Uuid) -> String {
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm_id,
            "realm-writer",
            "password",
            Some("realm-writer@example.com"),
            false,
        )
        .await
        .expect("create realm writer");

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
        .expect("assign role");

    let (login, _) = ctx
        .app_state
        .auth_service
        .create_session(&user, None, None, None)
        .await
        .expect("create session");
    login.access_token
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

async fn ensure_oauth_idp_browser_flow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            {
                "id": "auth-oauth",
                "type": "core.auth.oauth_idp",
                "data": { "config": { "auth_type": "core.auth.oauth_idp" } }
            },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } },
            { "id": "deny", "type": "core.terminal.deny", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-oauth", "source": "start", "target": "auth-oauth", "sourceHandle": "next" },
            { "id": "e-oauth-allow", "source": "auth-oauth", "target": "allow", "sourceHandle": "logged_in" },
            { "id": "e-oauth-jit", "source": "auth-oauth", "target": "allow", "sourceHandle": "jit_provisioned" },
            { "id": "e-oauth-deny", "source": "auth-oauth", "target": "deny", "sourceHandle": "failed" }
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

async fn ensure_collect_idp_choice_browser_flow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            {
                "id": "auth-collect",
                "type": "core.auth.collect_idp_choice",
                "data": { "config": { "auth_type": "core.auth.collect_idp_choice" } }
            },
            {
                "id": "auth-oauth",
                "type": "core.auth.oauth_idp",
                "data": { "config": { "auth_type": "core.auth.oauth_idp" } }
            },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } },
            { "id": "deny", "type": "core.terminal.deny", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-collect", "source": "start", "target": "auth-collect", "sourceHandle": "next" },
            { "id": "e-collect-oauth", "source": "auth-collect", "target": "auth-oauth", "sourceHandle": "selected" },
            { "id": "e-collect-deny", "source": "auth-collect", "target": "deny", "sourceHandle": "failed" },
            { "id": "e-oauth-allow", "source": "auth-oauth", "target": "allow", "sourceHandle": "logged_in" },
            { "id": "e-oauth-jit", "source": "auth-oauth", "target": "allow", "sourceHandle": "jit_provisioned" },
            { "id": "e-oauth-deny", "source": "auth-oauth", "target": "deny", "sourceHandle": "failed" }
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

async fn create_identity_provider(
    ctx: &TestContext,
    realm: &Realm,
    alias: &str,
    display_name: &str,
    sort_order: i64,
    allow_login: bool,
) -> IdentityProviderResponse {
    enable_identity_brokering(ctx, realm).await;
    create_oauth_identity_provider(
        ctx,
        realm,
        ProviderFixtureSpec {
            base_url: "https://example.com".to_string(),
            alias: alias.to_string(),
            display_name: display_name.to_string(),
            sort_order,
            allow_login,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await
}

struct ProviderFixtureSpec {
    base_url: String,
    alias: String,
    display_name: String,
    sort_order: i64,
    allow_login: bool,
    allow_link: bool,
    allow_email_auto_link: bool,
    enabled: bool,
}

async fn create_oauth_identity_provider(
    ctx: &TestContext,
    realm: &Realm,
    spec: ProviderFixtureSpec,
) -> IdentityProviderResponse {
    enable_identity_brokering(ctx, realm).await;
    ctx.app_state
        .identity_provider_service
        .create(
            realm.id,
            CreateIdentityProviderRequest {
                preset: None,
                alias: spec.alias.clone(),
                display_name: spec.display_name,
                protocol: IdentityProviderProtocol::Oauth2,
                client_id: format!("client-{}", spec.alias),
                client_secret: Some("secret".to_string()),
                issuer: None,
                authorization_endpoint: Some(format!("{}/authorize", spec.base_url)),
                token_endpoint: Some(format!("{}/token", spec.base_url)),
                userinfo_endpoint: Some(format!("{}/userinfo", spec.base_url)),
                jwks_uri: None,
                scopes: Some(vec!["openid".to_string(), "email".to_string()]),
                claim_mapping: Some(serde_json::json!({})),
                pkce_required: Some(true),
                allow_login: Some(spec.allow_login),
                allow_link: Some(spec.allow_link),
                allow_jit_provisioning: Some(false),
                allow_email_auto_link: Some(spec.allow_email_auto_link),
                require_verified_email: Some(true),
                icon_ref: None,
                button_color: None,
                sort_order: Some(spec.sort_order),
                enabled: Some(spec.enabled),
            },
        )
        .await
        .expect("create identity provider")
}

async fn create_github_identity_provider(
    ctx: &TestContext,
    realm: &Realm,
    spec: ProviderFixtureSpec,
) -> IdentityProviderResponse {
    enable_identity_brokering(ctx, realm).await;
    ctx.app_state
        .identity_provider_service
        .create(
            realm.id,
            CreateIdentityProviderRequest {
                preset: Some("github".to_string()),
                alias: spec.alias,
                display_name: spec.display_name,
                protocol: IdentityProviderProtocol::Oauth2,
                client_id: "client-github".to_string(),
                client_secret: Some("secret".to_string()),
                issuer: None,
                authorization_endpoint: Some(format!("{}/authorize", spec.base_url)),
                token_endpoint: Some(format!("{}/token", spec.base_url)),
                userinfo_endpoint: Some(format!("{}/user", spec.base_url)),
                jwks_uri: None,
                scopes: None,
                claim_mapping: Some(serde_json::json!({})),
                pkce_required: Some(true),
                allow_login: Some(spec.allow_login),
                allow_link: Some(spec.allow_link),
                allow_jit_provisioning: Some(false),
                allow_email_auto_link: Some(spec.allow_email_auto_link),
                require_verified_email: Some(true),
                icon_ref: None,
                button_color: None,
                sort_order: Some(spec.sort_order),
                enabled: Some(spec.enabled),
            },
        )
        .await
        .expect("create github identity provider")
}

async fn create_oidc_identity_provider(
    ctx: &TestContext,
    realm: &Realm,
    spec: ProviderFixtureSpec,
) -> IdentityProviderResponse {
    enable_identity_brokering(ctx, realm).await;
    ctx.app_state
        .identity_provider_service
        .create(
            realm.id,
            CreateIdentityProviderRequest {
                preset: None,
                alias: spec.alias.clone(),
                display_name: spec.display_name,
                protocol: IdentityProviderProtocol::Oidc,
                client_id: format!("client-{}", spec.alias),
                client_secret: Some("secret".to_string()),
                issuer: Some(spec.base_url.clone()),
                authorization_endpoint: Some(format!("{}/authorize", spec.base_url)),
                token_endpoint: Some(format!("{}/token", spec.base_url)),
                userinfo_endpoint: Some(format!("{}/userinfo", spec.base_url)),
                jwks_uri: Some(format!("{}/jwks", spec.base_url)),
                scopes: Some(vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ]),
                claim_mapping: Some(serde_json::json!({})),
                pkce_required: Some(true),
                allow_login: Some(spec.allow_login),
                allow_link: Some(spec.allow_link),
                allow_jit_provisioning: Some(false),
                allow_email_auto_link: Some(spec.allow_email_auto_link),
                require_verified_email: Some(true),
                icon_ref: None,
                button_color: None,
                sort_order: Some(spec.sort_order),
                enabled: Some(spec.enabled),
            },
        )
        .await
        .expect("create oidc identity provider")
}

async fn enable_identity_brokering(ctx: &TestContext, realm: &Realm) {
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
                idp_broker_enabled: Some(true),
                idp_default_jit_policy: None,
                idp_default_email_link_policy: None,
                idp_minimum_remaining_factor: None,
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
                invitation_flow_id: None,
            },
        )
        .await
        .expect("enable identity brokering");
}

async fn link_federated_identity(
    ctx: &TestContext,
    realm_id: Uuid,
    user: &User,
    local_password: &str,
    alias: &str,
    subject: &str,
) -> (Uuid, Uuid) {
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

    let federated_identity_id =
        link_existing_provider_identity(ctx, realm_id, &provider, user, local_password, subject)
            .await;
    (provider.id, federated_identity_id)
}

async fn link_existing_provider_identity(
    ctx: &TestContext,
    realm_id: Uuid,
    provider: &IdentityProviderResponse,
    user: &User,
    local_password: &str,
    subject: &str,
) -> Uuid {
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
                external_email: user.email.clone(),
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

async fn ensure_password_force_reset_browser_flow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "auth-password", "type": "core.auth.password", "data": { "config": { "auth_type": "core.auth.password" } } },
            { "id": "auth-force-reset", "type": "core.auth.reset_password", "data": { "config": { "auth_type": "core.auth.reset_password" } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } }
        ],
        "edges": [
            { "id": "e-start-password", "source": "start", "target": "auth-password", "sourceHandle": "next" },
            { "id": "e-password-allow", "source": "auth-password", "target": "allow", "sourceHandle": "success" },
            { "id": "e-password-force-reset", "source": "auth-password", "target": "auth-force-reset", "sourceHandle": "force_reset" },
            { "id": "e-force-reset-allow", "source": "auth-force-reset", "target": "allow", "sourceHandle": "success" }
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

async fn attach_login_action_binding(ctx: &TestContext, realm: &Realm) {
    let binding = ctx
        .app_state
        .theme_service
        .resolve_binding(realm.id, None)
        .await
        .expect("resolve binding")
        .expect("theme binding");

    let mut draft = ctx
        .app_state
        .theme_service
        .get_draft(realm.id, binding.theme_id)
        .await
        .expect("theme draft");

    let login_node = draft
        .nodes
        .iter_mut()
        .find(|node| node.node_key == "login")
        .expect("login page");

    let nodes = match login_node.blueprint {
        serde_json::Value::Array(ref mut nodes) => nodes,
        serde_json::Value::Object(ref mut map) => map
            .get_mut("nodes")
            .and_then(serde_json::Value::as_array_mut)
            .expect("login nodes array"),
        _ => panic!("unexpected login blueprint"),
    };

    let mut updated = false;
    for node in nodes.iter_mut() {
        let obj = match node.as_object_mut() {
            Some(value) => value,
            None => continue,
        };
        let node_type = obj.get("type").and_then(|value| value.as_str());
        let component = obj.get("component").and_then(|value| value.as_str());
        if node_type == Some("Component") && component == Some("Button") {
            let props = obj
                .entry("props")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .expect("button props");
            props.insert(
                "actions".to_string(),
                serde_json::json!([
                    {
                        "trigger": "on_click",
                        "signal": {
                            "type": "submit_node",
                            "node_id": "auth-password",
                            "payload_map": {
                                "username": "inputs.username",
                                "password": "inputs.password"
                            }
                        }
                    }
                ]),
            );
            updated = true;
            break;
        }
    }

    assert!(updated, "login button not found for action binding");

    ctx.app_state
        .theme_service
        .save_draft(realm.id, binding.theme_id, draft)
        .await
        .expect("save theme draft");
}

async fn attach_login_call_subflow_binding(ctx: &TestContext, realm: &Realm, target_node_id: &str) {
    let binding = ctx
        .app_state
        .theme_service
        .resolve_binding(realm.id, None)
        .await
        .expect("resolve binding")
        .expect("theme binding");

    let mut draft = ctx
        .app_state
        .theme_service
        .get_draft(realm.id, binding.theme_id)
        .await
        .expect("theme draft");

    let login_node = draft
        .nodes
        .iter_mut()
        .find(|node| node.node_key == "login")
        .expect("login page");

    let nodes = match login_node.blueprint {
        serde_json::Value::Array(ref mut nodes) => nodes,
        serde_json::Value::Object(ref mut map) => map
            .get_mut("nodes")
            .and_then(serde_json::Value::as_array_mut)
            .expect("login nodes array"),
        _ => panic!("unexpected login blueprint"),
    };

    let mut updated = false;
    for node in nodes.iter_mut() {
        let obj = match node.as_object_mut() {
            Some(value) => value,
            None => continue,
        };
        let node_type = obj.get("type").and_then(|value| value.as_str());
        let component = obj.get("component").and_then(|value| value.as_str());
        if node_type == Some("Component") && component == Some("Button") {
            let props = obj
                .entry("props")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .expect("button props");
            props.insert(
                "actions".to_string(),
                serde_json::json!([
                    {
                        "trigger": "on_click",
                        "signal": {
                            "type": "call_subflow",
                            "node_id": target_node_id,
                            "payload_map": {}
                        }
                    }
                ]),
            );
            updated = true;
            break;
        }
    }

    assert!(
        updated,
        "login button not found for call_subflow action binding"
    );

    ctx.app_state
        .theme_service
        .save_draft(realm.id, binding.theme_id, draft)
        .await
        .expect("save theme draft");
}

async fn create_and_publish_flow(
    ctx: &TestContext,
    realm: &Realm,
    flow_type: &str,
    name: &str,
    graph: serde_json::Value,
) -> Uuid {
    let draft = ctx
        .app_state
        .flow_manager
        .create_draft(
            realm.id,
            CreateDraftRequest {
                name: name.to_string(),
                description: None,
                flow_type: flow_type.to_string(),
            },
        )
        .await
        .expect("create draft");

    ctx.app_state
        .flow_manager
        .update_draft(
            draft.id,
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
        .publish_flow(realm.id, draft.id)
        .await
        .expect("publish flow");

    draft.id
}

async fn ensure_step_up_failure_flow(ctx: &TestContext, realm: &Realm) {
    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "deny", "type": "core.terminal.deny", "data": { "config": { "is_failure": true } } }
        ],
        "edges": [
            { "id": "e-start-deny", "source": "start", "target": "deny", "sourceHandle": "next" }
        ]
    });

    let _ = create_and_publish_flow(ctx, realm, "step_up", "Step Up", graph).await;
}

async fn ensure_password_browser_flow_with_subflow(ctx: &TestContext, realm: &Realm) {
    let flow_id = realm
        .browser_flow_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("browser flow id");

    let graph = serde_json::json!({
        "nodes": [
            { "id": "start", "type": "core.start", "data": { "config": {} } },
            { "id": "auth-password", "type": "core.auth.password", "data": { "config": { "auth_type": "core.auth.password" } } },
            { "id": "call-step-up", "type": "core.logic.subflow", "data": { "config": { "logic_type": "core.logic.subflow", "flow_type": "step_up" } } },
            { "id": "allow", "type": "core.terminal.allow", "data": { "config": {} } },
            { "id": "deny", "type": "core.terminal.deny", "data": { "config": { "is_failure": true } } }
        ],
        "edges": [
            { "id": "e-start-password", "source": "start", "target": "auth-password", "sourceHandle": "next" },
            { "id": "e-password-allow", "source": "auth-password", "target": "allow", "sourceHandle": "success" },
            { "id": "e-password-step-up", "source": "auth-password", "target": "call-step-up", "sourceHandle": "failure" },
            { "id": "e-step-up-allow", "source": "call-step-up", "target": "allow", "sourceHandle": "success" },
            { "id": "e-step-up-deny", "source": "call-step-up", "target": "deny", "sourceHandle": "failure" }
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

    let _ = ctx
        .app_state
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
        .create_user(realm.id, "alice", "password-123", None, false)
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
        .create_user(realm.id, "bob", "password-123", None, false)
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
        .create_user(realm.id, "charlie", "password-123", None, false)
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
async fn auth_login_challenge_includes_enabled_identity_providers() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    create_identity_provider(&ctx, &realm, "microsoft", "Microsoft", 20, true).await;
    create_identity_provider(&ctx, &realm, "github", "GitHub", 10, true).await;
    create_identity_provider(&ctx, &realm, "hidden", "Hidden", 5, false).await;

    let mut request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("challenge json");

    assert_eq!(
        json.get("challengeName").and_then(|value| value.as_str()),
        Some("login-password")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("enabled_providers_count"))
            .and_then(|value| value.as_u64()),
        Some(2)
    );
    let aliases = json
        .get("context")
        .and_then(|value| value.get("enabled_providers"))
        .and_then(|value| value.as_array())
        .expect("enabled providers array")
        .iter()
        .filter_map(|value| value.get("alias").and_then(|value| value.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(aliases, vec!["github", "microsoft"]);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_challenge_hides_identity_providers_when_brokering_disabled() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    create_identity_provider(&ctx, &realm, "github", "GitHub", 10, true).await;

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
                idp_broker_enabled: Some(false),
                idp_default_jit_policy: None,
                idp_default_email_link_policy: None,
                idp_minimum_remaining_factor: None,
                browser_flow_id: None,
                registration_flow_id: None,
                direct_grant_flow_id: None,
                reset_credentials_flow_id: None,
                invitation_flow_id: None,
            },
        )
        .await
        .expect("disable identity brokering");

    let mut request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("challenge json");

    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("enabled_providers_count"))
            .and_then(|value| value.as_u64()),
        Some(0)
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("enabled_providers"))
            .and_then(|value| value.as_array())
            .map(|providers| providers.len()),
        Some(0)
    );
}

#[tokio::test]
#[serial(test_db)]
async fn collect_idp_choice_flow_advances_to_oauth_idp_with_selected_provider() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    create_identity_provider(&ctx, &realm, "microsoft", "Microsoft", 20, true).await;
    create_identity_provider(&ctx, &realm, "github", "GitHub", 10, true).await;
    create_identity_provider(&ctx, &realm, "hidden", "Hidden", 5, false).await;
    ensure_collect_idp_choice_browser_flow(&ctx, &realm).await;

    let mut request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let session_id = cookie_value(response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("challenge json");
    assert_eq!(
        json.get("challengeName").and_then(|value| value.as_str()),
        Some("core.auth.collect_idp_choice")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_select")
    );
    let aliases = json
        .get("context")
        .and_then(|value| value.get("enabled_providers"))
        .and_then(|value| value.as_array())
        .expect("enabled providers array")
        .iter()
        .filter_map(|value| value.get("alias").and_then(|value| value.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(aliases, vec!["github", "microsoft"]);

    let payload = serde_json::json!({
        "provider_alias": "microsoft"
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
    let exec_body = exec_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let exec_json: serde_json::Value =
        serde_json::from_slice(&exec_body).expect("execute challenge json");
    assert_eq!(
        exec_json
            .get("challengeName")
            .and_then(|value| value.as_str()),
        Some("core.auth.oauth_idp")
    );
    assert_eq!(
        exec_json
            .get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_redirecting")
    );
    assert_eq!(
        exec_json
            .get("context")
            .and_then(|value| value.get("provider_alias"))
            .and_then(|value| value.as_str()),
        Some("microsoft")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn identity_provider_delete_soft_disables_when_links_exist() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "provider-linked-user",
            "password-123",
            Some("provider-linked@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let (provider_id, federated_identity_id) = link_federated_identity(
        &ctx,
        realm.id,
        &target_user,
        "password-123",
        "github-linked",
        "linked-subject-1",
    )
    .await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/identity-providers/{}",
            DEFAULT_REALM_NAME, provider_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("delete request");
    let delete_response = ctx.request(delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let delete_body = delete_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let delete_json: serde_json::Value = serde_json::from_slice(&delete_body).expect("delete json");
    assert_eq!(
        delete_json.get("outcome").and_then(|value| value.as_str()),
        Some("soft_deleted")
    );
    assert_eq!(
        delete_json
            .get("linked_identity_count")
            .and_then(|value| value.as_u64()),
        Some(1)
    );

    let provider_after = ctx
        .app_state
        .identity_provider_service
        .get_by_id(provider_id)
        .await
        .expect("provider after delete");
    assert!(!provider_after.enabled);
    assert!(!provider_after.allow_login);
    assert!(!provider_after.allow_link);

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, target_user.id)
        .await
        .expect("credentials after soft delete");
    assert!(credentials
        .federated_identities
        .iter()
        .any(|identity| identity.id == federated_identity_id));
}

#[tokio::test]
#[serial(test_db)]
async fn identity_provider_delete_hard_deletes_provider_and_links() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let target_user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "provider-hard-delete-user",
            "password-123",
            Some("provider-hard-delete@example.com"),
            false,
        )
        .await
        .expect("create target user");

    let (provider_id, _federated_identity_id) = link_federated_identity(
        &ctx,
        realm.id,
        &target_user,
        "password-123",
        "google-linked",
        "linked-subject-2",
    )
    .await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/realms/{}/identity-providers/{}?hard=true",
            DEFAULT_REALM_NAME, provider_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("delete request");
    let delete_response = ctx.request(delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let delete_body = delete_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let delete_json: serde_json::Value = serde_json::from_slice(&delete_body).expect("delete json");
    assert_eq!(
        delete_json.get("outcome").and_then(|value| value.as_str()),
        Some("hard_deleted")
    );
    assert_eq!(
        delete_json
            .get("linked_identity_count")
            .and_then(|value| value.as_u64()),
        Some(1)
    );

    let provider_after = ctx
        .app_state
        .identity_provider_service
        .get_by_id(provider_id)
        .await;
    assert!(provider_after.is_err());

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, target_user.id)
        .await
        .expect("credentials after hard delete");
    assert!(credentials.federated_identities.is_empty());
}

#[tokio::test]
#[serial(test_db)]
async fn identity_provider_test_connection_refreshes_metadata_and_jwks() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let upstream = FakeOauthUpstream::start().await;
    let provider = create_oidc_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "oidc-test".to_string(),
            display_name: "OIDC Test".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/realms/{}/identity-providers/{}/test-connection",
            DEFAULT_REALM_NAME, provider.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("test request");
    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("test json");

    assert_eq!(json.get("ok").and_then(|value| value.as_bool()), Some(true));
    assert_eq!(
        json.get("discovery")
            .and_then(|value| value.get("ok"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        json.get("token_endpoint")
            .and_then(|value| value.get("status_code"))
            .and_then(|value| value.as_u64()),
        Some(400)
    );
    assert_eq!(
        json.get("userinfo_endpoint")
            .and_then(|value| value.get("status_code"))
            .and_then(|value| value.as_u64()),
        Some(401)
    );
    assert_eq!(
        json.get("jwks")
            .and_then(|value| value.get("ok"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );

    let provider_after = ctx
        .app_state
        .identity_provider_service
        .get_by_id(provider.id)
        .await
        .expect("provider after test");
    assert!(provider_after.metadata_cached_at.is_some());
    assert!(provider_after.jwks_cached_at.is_some());
}

#[tokio::test]
#[serial(test_db)]
async fn identity_provider_linked_users_inventory_returns_linked_accounts() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let provider =
        create_identity_provider(&ctx, &realm, "github-linked-users", "GitHub", 10, true).await;

    let user_one = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "linked-user-one",
            "password-123",
            Some("linked-one@example.com"),
            false,
        )
        .await
        .expect("create first user");
    let user_two = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "linked-user-two",
            "password-123",
            Some("linked-two@example.com"),
            false,
        )
        .await
        .expect("create second user");

    link_existing_provider_identity(
        &ctx,
        realm.id,
        &provider,
        &user_one,
        "password-123",
        "linked-subject-one",
    )
    .await;
    link_existing_provider_identity(
        &ctx,
        realm.id,
        &provider,
        &user_two,
        "password-123",
        "linked-subject-two",
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/identity-providers/{}/linked-users",
            DEFAULT_REALM_NAME, provider.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("linked users request");
    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("linked users json");
    let linked_users = json.as_array().expect("linked users array");
    assert_eq!(linked_users.len(), 2);

    let usernames = linked_users
        .iter()
        .filter_map(|value| value.get("username").and_then(|value| value.as_str()))
        .collect::<Vec<_>>();
    assert!(usernames.contains(&"linked-user-one"));
    assert!(usernames.contains(&"linked-user-two"));

    let subjects = linked_users
        .iter()
        .filter_map(|value| value.get("subject").and_then(|value| value.as_str()))
        .collect::<Vec<_>>();
    assert!(subjects.contains(&"linked-subject-one"));
    assert!(subjects.contains(&"linked-subject-two"));
}

#[tokio::test]
#[serial(test_db)]
async fn identity_provider_activity_returns_recent_broker_events_for_provider() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let provider =
        create_identity_provider(&ctx, &realm, "github-activity", "GitHub", 10, true).await;
    let other_provider =
        create_identity_provider(&ctx, &realm, "github-other", "GitHub Other", 20, true).await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "activity-user",
            "password-123",
            Some("activity@example.com"),
            false,
        )
        .await
        .expect("create activity user");
    let auth_session_id = Uuid::new_v4();

    ctx.app_state
        .audit_service
        .record(NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: None,
            action: "idp_callback_failure".to_string(),
            target_type: "identity_provider".to_string(),
            target_id: Some(provider.id.to_string()),
            metadata: serde_json::json!({
                "provider_alias": provider.alias,
                "auth_session_id": auth_session_id,
                "message": "Token exchange failed with status 502"
            }),
        })
        .await
        .expect("record callback failure");
    ctx.app_state
        .audit_service
        .record(NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: Some(user.id),
            action: "idp_user_linked".to_string(),
            target_type: "identity_provider".to_string(),
            target_id: Some(provider.id.to_string()),
            metadata: serde_json::json!({
                "provider_alias": provider.alias,
                "auth_session_id": auth_session_id,
                "user_id": user.id,
                "linked_via": "manual",
                "subject": "activity-subject"
            }),
        })
        .await
        .expect("record user linked");
    ctx.app_state
        .audit_service
        .record(NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: None,
            action: "idp_callback_failure".to_string(),
            target_type: "identity_provider".to_string(),
            target_id: Some(other_provider.id.to_string()),
            metadata: serde_json::json!({
                "provider_alias": other_provider.alias,
                "auth_session_id": Uuid::new_v4(),
                "message": "Other provider failure"
            }),
        })
        .await
        .expect("record unrelated provider event");
    ctx.app_state
        .audit_service
        .record(NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: None,
            action: "user.login".to_string(),
            target_type: "user".to_string(),
            target_id: Some(user.id.to_string()),
            metadata: serde_json::json!({}),
        })
        .await
        .expect("record unrelated audit event");

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/realms/{}/identity-providers/{}/activity?limit=10",
            DEFAULT_REALM_NAME, provider.id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("activity request");
    let response = ctx.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("activity json");
    assert_eq!(
        json.get("provider_alias").and_then(|value| value.as_str()),
        Some("github-activity")
    );
    assert_eq!(
        json.get("summary")
            .and_then(|value| value.get("failures_last_24h"))
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert_eq!(
        json.get("summary")
            .and_then(|value| value.get("links_last_24h"))
            .and_then(|value| value.as_u64()),
        Some(1)
    );

    let recent_events = json
        .get("recent_events")
        .and_then(|value| value.as_array())
        .expect("recent events array");
    let auth_session_id_string = auth_session_id.to_string();
    assert_eq!(recent_events.len(), 2);
    assert!(recent_events.iter().all(|event| {
        event
            .get("auth_session_id")
            .and_then(|value| value.as_str())
            == Some(auth_session_id_string.as_str())
    }));
    assert!(recent_events.iter().any(|event| {
        event.get("action").and_then(|value| value.as_str()) == Some("idp_callback_failure")
    }));
    assert!(recent_events.iter().any(|event| {
        event.get("action").and_then(|value| value.as_str()) == Some("idp_user_linked")
    }));
}

#[tokio::test]
#[serial(test_db)]
async fn realm_idp_settings_get_and_update_round_trip() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let get_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("/api/realms/{}/idp-settings", realm.id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .expect("get request"),
        )
        .await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = get_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let get_json: serde_json::Value = serde_json::from_slice(&get_body).expect("settings json");
    assert_eq!(
        get_json
            .get("oauth_start_rate_limit_max")
            .and_then(|value| value.as_i64()),
        Some(30)
    );
    assert_eq!(
        get_json
            .get("oauth_start_rate_limit_window_minutes")
            .and_then(|value| value.as_i64()),
        Some(10)
    );

    let update_response = ctx
        .request(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/realms/{}/idp-settings", realm.id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "oauth_start_rate_limit_max": 12,
                        "oauth_start_rate_limit_window_minutes": 7
                    })
                    .to_string(),
                ))
                .expect("update request"),
        )
        .await;
    assert_eq!(update_response.status(), StatusCode::OK);

    let updated = ctx
        .app_state
        .realm_idp_settings_service
        .get_settings(realm.id)
        .await
        .expect("fetch settings");
    assert_eq!(updated.oauth_start_rate_limit_max, 12);
    assert_eq!(updated.oauth_start_rate_limit_window_minutes, 7);
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_start_returns_authorization_redirect_with_state_and_pkce() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(start_response.status(), StatusCode::OK);
    let body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("start json");
    let redirect_url = json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let parsed = Url::parse(redirect_url).expect("redirect parse");
    assert_eq!(parsed.path(), "/authorize");
    let query = parsed.query_pairs().into_owned().collect::<HashMap<_, _>>();
    assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
    assert_eq!(
        query.get("client_id").map(String::as_str),
        Some("client-github")
    );
    assert!(query.contains_key("state"));
    assert!(query.contains_key("code_challenge"));
    assert_eq!(
        query.get("code_challenge_method").map(String::as_str),
        Some("S256")
    );

    let state_id = query
        .get("state")
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("state id");
    let session = ctx
        .app_state
        .auth_session_repo
        .find_by_id(&session_id)
        .await
        .expect("session lookup")
        .expect("session missing");
    assert!(
        session
            .context
            .get("oauth_broker_verifiers")
            .and_then(|value| value.get(state_id.to_string()))
            .and_then(|value| value.as_str())
            .is_some(),
        "oauth verifier should be stored in session context"
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_start_is_rate_limited_per_provider_and_emits_activity_event() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let provider =
        create_identity_provider(&ctx, &realm, "github-throttled", "GitHub", 10, true).await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    ctx.app_state
        .realm_idp_settings_service
        .update_settings(
            realm.id,
            UpdateRealmIdpSettingsPayload {
                oauth_start_rate_limit_max: Some(2),
                oauth_start_rate_limit_window_minutes: Some(10),
            },
        )
        .await
        .expect("update idp settings");

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    for _ in 0..2 {
        let response = ctx
            .request(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/realms/{}/auth/oauth/{}/start",
                        DEFAULT_REALM_NAME, provider.alias
                    ))
                    .header(
                        header::COOKIE,
                        format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                    )
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::empty())
                    .expect("start request"),
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    let limited_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/start",
                    DEFAULT_REALM_NAME, provider.alias
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .header("x-forwarded-for", "203.0.113.10")
                .body(Body::empty())
                .expect("limited request"),
        )
        .await;
    assert_eq!(limited_response.status(), StatusCode::TOO_MANY_REQUESTS);

    let limited_body = limited_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let limited_json: serde_json::Value =
        serde_json::from_slice(&limited_body).expect("limited json");
    assert_eq!(
        limited_json.get("code").and_then(|value| value.as_str()),
        Some("request.rate_limited")
    );

    let other_ip_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/start",
                    DEFAULT_REALM_NAME, provider.alias
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .header("x-forwarded-for", "203.0.113.11")
                .body(Body::empty())
                .expect("other ip request"),
        )
        .await;
    assert_eq!(other_ip_response.status(), StatusCode::OK);

    let activity_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/identity-providers/{}/activity?limit=10",
                    DEFAULT_REALM_NAME, provider.id
                ))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .expect("activity request"),
        )
        .await;
    assert_eq!(activity_response.status(), StatusCode::OK);

    let activity_body = activity_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let activity_json: serde_json::Value =
        serde_json::from_slice(&activity_body).expect("activity json");
    assert_eq!(
        activity_json
            .get("summary")
            .and_then(|value| value.get("failures_last_24h"))
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert!(activity_json
        .get("recent_events")
        .and_then(|value| value.as_array())
        .expect("recent events array")
        .iter()
        .any(|event| {
            event.get("action").and_then(|value| value.as_str()) == Some("idp_start_rate_limited")
        }));
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_missing_state_records_invalid_request_activity() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let provider =
        create_identity_provider(&ctx, &realm, "github-invalid", "GitHub Invalid", 10, true).await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    let response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/callback?code=missing-state",
                    DEFAULT_REALM_NAME, provider.alias
                ))
                .body(Body::empty())
                .expect("callback request"),
        )
        .await;
    assert_eq!(response.status(), StatusCode::FOUND);

    let activity_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/identity-providers/{}/activity?limit=10",
                    DEFAULT_REALM_NAME, provider.id
                ))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .expect("activity request"),
        )
        .await;
    assert_eq!(activity_response.status(), StatusCode::OK);

    let body = activity_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("activity json");
    assert_eq!(
        json.get("summary")
            .and_then(|value| value.get("failures_last_24h"))
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert!(json
        .get("recent_events")
        .and_then(|value| value.as_array())
        .expect("recent events")
        .iter()
        .any(|event| {
            event.get("action").and_then(|value| value.as_str())
                == Some("idp_callback_invalid_request")
                && event
                    .get("metadata")
                    .and_then(|value| value.get("reason"))
                    .and_then(|value| value.as_str())
                    == Some("missing_state")
        }));
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_upstream_error_records_specific_activity_event() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let upstream = FakeOauthUpstream::start().await;
    let provider = create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github-upstream-error".to_string(),
            display_name: "GitHub Upstream Error".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/start",
                    DEFAULT_REALM_NAME, provider.alias
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/callback?error=access_denied&state={}",
                    DEFAULT_REALM_NAME, provider.alias, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .expect("callback request"),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);

    let activity_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/identity-providers/{}/activity?limit=10",
                    DEFAULT_REALM_NAME, provider.id
                ))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .expect("activity request"),
        )
        .await;
    assert_eq!(activity_response.status(), StatusCode::OK);

    let body = activity_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("activity json");
    assert!(json
        .get("recent_events")
        .and_then(|value| value.as_array())
        .expect("recent events")
        .iter()
        .any(|event| {
            event.get("action").and_then(|value| value.as_str())
                == Some("idp_callback_upstream_error")
                && event
                    .get("metadata")
                    .and_then(|value| value.get("upstream_error"))
                    .and_then(|value| value.as_str())
                    == Some("access_denied")
        }));
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_session_mismatch_records_specific_activity_event() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    let token = setup_realm_writer_token(&ctx, realm.id).await;

    let upstream = FakeOauthUpstream::start().await;
    let provider = create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github-session-mismatch".to_string(),
            display_name: "GitHub Session Mismatch".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: true,
            enabled: true,
        },
    )
    .await;
    let _user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "session-mismatch-user",
            "password-123",
            Some("session-mismatch@example.com"),
            false,
        )
        .await
        .expect("create user");
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_userinfo(
            "session-mismatch",
            serde_json::json!({
                "sub": "github-session-mismatch-user",
                "email": "session-mismatch@example.com",
                "email_verified": true,
                "login": "session-mismatch-user"
            }),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/start",
                    DEFAULT_REALM_NAME, provider.alias
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/{}/callback?code=session-mismatch&state={}",
                    DEFAULT_REALM_NAME, provider.alias, state
                ))
                .body(Body::empty())
                .expect("callback request"),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);

    let activity_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/identity-providers/{}/activity?limit=10",
                    DEFAULT_REALM_NAME, provider.id
                ))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .expect("activity request"),
        )
        .await;
    assert_eq!(activity_response.status(), StatusCode::OK);

    let body = activity_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("activity json");
    assert!(json
        .get("recent_events")
        .and_then(|value| value.as_array())
        .expect("recent events")
        .iter()
        .any(|event| {
            event.get("action").and_then(|value| value.as_str())
                == Some("idp_callback_session_mismatch")
        }));
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_success_auto_links_and_redirects_home() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "oauth-user",
            "password-123",
            Some("oauth-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: true,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_userinfo(
            "success",
            serde_json::json!({
                "sub": "github-user-1",
                "email": "oauth-user@example.com",
                "email_verified": true,
                "preferred_username": "oauth-user"
            }),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=success&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/")
    );
    assert!(cookie_present(
        callback_response.headers(),
        REFRESH_TOKEN_COOKIE
    ));

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, user.id)
        .await
        .expect("list credentials");
    assert_eq!(credentials.federated_identities.len(), 1);
    assert_eq!(
        credentials.federated_identities[0].provider_alias,
        "github".to_string()
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_from_standard_login_provider_button_auto_links_and_redirects_home() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "standard-oauth-user",
            "password-123",
            Some("standard-oauth-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    create_github_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: true,
            enabled: true,
        },
    )
    .await;
    ensure_password_browser_flow(&ctx, &realm).await;
    upstream
        .set_github_profile(
            "standard-success",
            serde_json::json!({
                "id": 4242,
                "login": "standard-oauth-user",
                "email": null
            }),
            serde_json::json!([
                {
                    "email": "standard-oauth-user@example.com",
                    "primary": true,
                    "verified": true
                }
            ]),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=standard-success&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/")
    );
    assert!(cookie_present(
        callback_response.headers(),
        REFRESH_TOKEN_COOKIE
    ));

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, user.id)
        .await
        .expect("list credentials");
    assert_eq!(credentials.federated_identities.len(), 1);
    assert_eq!(
        credentials.federated_identities[0]
            .external_email
            .as_deref(),
        Some("standard-oauth-user@example.com")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn github_callback_uses_emails_endpoint_for_canonical_email() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "github-email-user",
            "password-123",
            Some("github-email-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    create_github_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: true,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_github_profile(
            "github-email",
            serde_json::json!({
                "id": 42,
                "login": "github-email-user",
                "email": null
            }),
            serde_json::json!([
                {
                    "email": "github-email-user@example.com",
                    "primary": true,
                    "verified": true
                }
            ]),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=github-email&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/")
    );
    assert!(cookie_present(
        callback_response.headers(),
        REFRESH_TOKEN_COOKIE
    ));

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, user.id)
        .await
        .expect("list credentials");
    assert_eq!(credentials.federated_identities.len(), 1);
    assert_eq!(
        credentials.federated_identities[0]
            .external_email
            .as_deref(),
        Some("github-email-user@example.com")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_callback_validates_id_token_and_caches_jwks() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "oidc-user",
            "password-123",
            Some("oidc-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    let provider = create_oidc_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "google".to_string(),
            display_name: "Google".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: true,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/google/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let redirect = Url::parse(redirect_url).expect("redirect url parse");
    let state = redirect
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");
    let nonce = redirect
        .query_pairs()
        .find(|(key, _)| key == "nonce")
        .map(|(_, value)| value.to_string())
        .expect("nonce");

    let id_token = issue_test_id_token(
        &upstream.base_url,
        &provider.client_id,
        "oidc-subject-1",
        &nonce,
        serde_json::json!({
            "email": "oidc-user@example.com",
            "email_verified": true
        }),
    );
    upstream
        .set_oidc_userinfo(
            "oidc-success",
            serde_json::json!({
                "sub": "oidc-subject-1",
                "preferred_username": "oidc-user"
            }),
            id_token,
        )
        .await;

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/google/callback?code=oidc-success&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/")
    );
    assert!(cookie_present(
        callback_response.headers(),
        REFRESH_TOKEN_COOKIE
    ));

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, user.id)
        .await
        .expect("list credentials");
    assert_eq!(credentials.federated_identities.len(), 1);
    assert_eq!(
        credentials.federated_identities[0].provider_alias,
        "google".to_string()
    );

    let stored_provider = ctx
        .app_state
        .identity_provider_service
        .get_domain_by_alias(realm.id, "google")
        .await
        .expect("stored provider");
    assert!(stored_provider.jwks_cached_at.is_some());
    assert!(stored_provider.jwks_cache_json.is_some());
}

#[tokio::test]
#[serial(test_db)]
async fn oidc_callback_rejects_nonce_mismatch_and_resumes_oauth_failure() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let provider = create_oidc_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "google".to_string(),
            display_name: "Google".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/google/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let id_token = issue_test_id_token(
        &upstream.base_url,
        &provider.client_id,
        "oidc-subject-2",
        "wrong-nonce",
        serde_json::json!({
            "email": "nonce-user@example.com",
            "email_verified": true
        }),
    );
    upstream
        .set_oidc_userinfo(
            "oidc-bad-nonce",
            serde_json::json!({
                "sub": "oidc-subject-2",
                "preferred_username": "nonce-user"
            }),
            id_token,
        )
        .await;

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/google/callback?code=oidc-bad-nonce&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/#/login?realm=master")
    );

    let mut resume_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::empty())
        .unwrap();
    resume_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let resume_response = ctx.request(resume_request).await;
    assert_eq!(resume_response.status(), StatusCode::OK);
    let body = resume_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("resume json");
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_failure")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("message"))
            .and_then(|value| value.as_str()),
        Some("Validation failed: OIDC id_token nonce mismatch")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_conflict_redirects_back_to_oauth_conflict_page() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "conflict-user",
            "password-123",
            Some("conflict-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: false,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_userinfo(
            "conflict",
            serde_json::json!({
                "sub": "github-conflict-1",
                "email": "conflict-user@example.com",
                "email_verified": true,
                "preferred_username": "conflict-user"
            }),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=conflict&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/#/login?realm=master")
    );
    let resumed_session_id = cookie_value(callback_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("resumed login session cookie");
    assert_eq!(resumed_session_id, session_id);

    let mut resume_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, resumed_session_id),
        )
        .body(Body::empty())
        .unwrap();
    resume_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let resume_response = ctx.request(resume_request).await;
    assert_eq!(resume_response.status(), StatusCode::OK);
    let body = resume_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("resume json");
    assert_eq!(
        json.get("challengeName").and_then(|value| value.as_str()),
        Some("core.auth.oauth_idp")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_conflict")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("external_email"))
            .and_then(|value| value.as_str()),
        user.email.as_deref()
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_manual_link_reprompts_on_failure_then_succeeds() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let user = ctx
        .app_state
        .user_service
        .create_user(
            realm.id,
            "manual-user",
            "password-123",
            Some("manual-user@example.com"),
            false,
        )
        .await
        .expect("create user");
    create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_userinfo(
            "manual-link",
            serde_json::json!({
                "sub": "github-manual-1",
                "email": "manual-user@example.com",
                "email_verified": false,
                "preferred_username": "manual-user"
            }),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=manual-link&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);

    let mut resume_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::empty())
        .unwrap();
    resume_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let resume_response = ctx.request(resume_request).await;
    let resume_body = resume_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let resume_json: serde_json::Value = serde_json::from_slice(&resume_body).expect("resume json");
    assert_eq!(
        resume_json
            .get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_link_confirm")
    );

    let wrong_password_response = ctx
        .request({
            let mut request = Request::builder()
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
                .body(Body::from(
                    serde_json::json!({
                        "username": user.username,
                        "password": "wrong-password"
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
    assert_eq!(wrong_password_response.status(), StatusCode::OK);
    let wrong_password_body = wrong_password_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let wrong_password_json: serde_json::Value =
        serde_json::from_slice(&wrong_password_body).expect("wrong password json");
    assert_eq!(
        wrong_password_json
            .get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_link_confirm")
    );
    assert_eq!(
        wrong_password_json
            .get("context")
            .and_then(|value| value.get("error"))
            .and_then(|value| value.as_str()),
        Some("The credentials provided are incorrect.")
    );

    let success_response = ctx
        .request({
            let mut request = Request::builder()
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
                .body(Body::from(
                    serde_json::json!({
                        "username": user.username,
                        "password": "password-123"
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
    assert_eq!(success_response.status(), StatusCode::OK);
    let success_body = success_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let success_json: serde_json::Value =
        serde_json::from_slice(&success_body).expect("success json");
    assert_eq!(
        success_json.get("status").and_then(|value| value.as_str()),
        Some("redirect")
    );
    assert_eq!(
        success_json.get("url").and_then(|value| value.as_str()),
        Some("/")
    );

    let credentials = ctx
        .app_state
        .user_credentials_service
        .list_credentials(realm.id, user.id)
        .await
        .expect("list credentials");
    assert_eq!(credentials.federated_identities.len(), 1);
    assert_eq!(
        credentials.federated_identities[0].provider_alias,
        "github".to_string()
    );
}

#[tokio::test]
#[serial(test_db)]
async fn oauth_callback_disabled_provider_mid_flow_surfaces_oauth_failure() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;

    let upstream = FakeOauthUpstream::start().await;
    let provider = create_oauth_identity_provider(
        &ctx,
        &realm,
        ProviderFixtureSpec {
            base_url: upstream.base_url.clone(),
            alias: "github".to_string(),
            display_name: "GitHub".to_string(),
            sort_order: 10,
            allow_login: true,
            allow_link: true,
            allow_email_auto_link: false,
            enabled: true,
        },
    )
    .await;
    ensure_oauth_idp_browser_flow(&ctx, &realm).await;
    upstream
        .set_userinfo(
            "disabled",
            serde_json::json!({
                "sub": "github-disabled-1",
                "email": "disabled-user@example.com",
                "email_verified": true,
                "preferred_username": "disabled-user"
            }),
        )
        .await;

    let mut login_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .body(Body::empty())
        .unwrap();
    login_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let login_response = ctx.request(login_request).await;
    let session_id = cookie_value(login_response.headers(), LOGIN_SESSION_COOKIE)
        .and_then(|value| Uuid::parse_str(&value).ok())
        .expect("login session cookie");

    let start_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/start",
                    DEFAULT_REALM_NAME
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    let start_body = start_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let start_json: serde_json::Value = serde_json::from_slice(&start_body).expect("start json");
    let redirect_url = start_json
        .get("redirect_url")
        .and_then(|value| value.as_str())
        .expect("redirect url");
    let state = Url::parse(redirect_url)
        .expect("redirect url parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("state");

    ctx.app_state
        .identity_provider_service
        .update(
            provider.id,
            UpdateIdentityProviderRequest {
                alias: None,
                display_name: None,
                client_id: None,
                client_secret: None,
                issuer: None,
                authorization_endpoint: None,
                token_endpoint: None,
                userinfo_endpoint: None,
                jwks_uri: None,
                scopes: None,
                claim_mapping: None,
                pkce_required: None,
                allow_login: None,
                allow_link: None,
                allow_jit_provisioning: None,
                allow_email_auto_link: None,
                require_verified_email: None,
                icon_ref: None,
                button_color: None,
                sort_order: None,
                enabled: Some(false),
            },
        )
        .await
        .expect("disable provider");

    let callback_response = ctx
        .request(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/realms/{}/auth/oauth/github/callback?code=disabled&state={}",
                    DEFAULT_REALM_NAME, state
                ))
                .header(
                    header::COOKIE,
                    format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(callback_response.status(), StatusCode::FOUND);
    assert_eq!(
        callback_response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/#/login?realm=master")
    );

    let mut resume_request = Request::builder()
        .method("GET")
        .uri(format!("/api/realms/{}/auth/login", DEFAULT_REALM_NAME))
        .header(
            header::COOKIE,
            format!("{}={}", LOGIN_SESSION_COOKIE, session_id),
        )
        .body(Body::empty())
        .unwrap();
    resume_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
    let resume_response = ctx.request(resume_request).await;
    assert_eq!(resume_response.status(), StatusCode::OK);
    let body = resume_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("resume json");
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("template_key"))
            .and_then(|value| value.as_str()),
        Some("oauth_failure")
    );
    assert_eq!(
        json.get("context")
            .and_then(|value| value.get("message"))
            .and_then(|value| value.as_str()),
        Some("Validation failed: Identity provider is unavailable")
    );
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_flow_forces_password_reset_when_flagged() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_password_force_reset_browser_flow(&ctx, &realm).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "force-reset-user", "password-123", None, false)
        .await
        .expect("create user");
    ctx.app_state
        .user_service
        .update_credential_policy(realm.id, user.id, Some(true), None)
        .await
        .expect("flag force reset");

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

    let password_payload = serde_json::json!({
        "username": user.username,
        "password": "password-123"
    });
    let mut password_exec_request = Request::builder()
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
        .body(Body::from(password_payload.to_string()))
        .unwrap();
    password_exec_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let password_exec_response = ctx.request(password_exec_request).await;
    assert_eq!(password_exec_response.status(), StatusCode::OK);
    let password_exec_body = password_exec_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let password_exec_json: serde_json::Value =
        serde_json::from_slice(&password_exec_body).expect("challenge json");
    assert_eq!(
        password_exec_json.get("status").and_then(|v| v.as_str()),
        Some("challenge")
    );
    assert_eq!(
        password_exec_json
            .get("challengeName")
            .and_then(|v| v.as_str()),
        Some("core.auth.reset_password")
    );

    let reset_payload = serde_json::json!({
        "password": "password-456",
        "password_confirm": "password-456"
    });
    let mut reset_exec_request = Request::builder()
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
        .body(Body::from(reset_payload.to_string()))
        .unwrap();
    reset_exec_request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));

    let reset_exec_response = ctx.request(reset_exec_request).await;
    assert_eq!(reset_exec_response.status(), StatusCode::OK);
    let reset_exec_json: serde_json::Value = serde_json::from_slice(
        &reset_exec_response
            .into_body()
            .collect()
            .await
            .expect("read body")
            .to_bytes(),
    )
    .expect("json");
    assert_eq!(
        reset_exec_json.get("status").and_then(|v| v.as_str()),
        Some("redirect")
    );

    let updated_user = ctx
        .app_state
        .user_service
        .get_user_in_realm(realm.id, user.id)
        .await
        .expect("updated user");
    assert!(!updated_user.force_password_reset);
}

#[tokio::test]
#[serial(test_db)]
async fn auth_login_flow_executes_signal_envelope() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    attach_login_action_binding(&ctx, &realm).await;
    ensure_password_browser_flow(&ctx, &realm).await;

    let user = ctx
        .app_state
        .user_service
        .create_user(realm.id, "signal-user", "password-123", None, false)
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
        "signal": {
            "type": "submit_node",
            "node_id": "auth-password",
            "payload": {
                "username": user.username,
                "password": "password-123"
            }
        }
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
async fn auth_login_flow_executes_call_subflow_signal_envelope() {
    let ctx = TestContext::new().await;
    let realm = setup_master_realm(&ctx).await;
    ensure_step_up_failure_flow(&ctx, &realm).await;
    ensure_password_browser_flow_with_subflow(&ctx, &realm).await;
    attach_login_call_subflow_binding(&ctx, &realm, "call-step-up").await;

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
        "signal": {
            "type": "call_subflow",
            "node_id": "call-step-up",
            "payload": {}
        }
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
    assert_eq!(exec_response.status(), StatusCode::UNAUTHORIZED);

    let exec_body = exec_response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let exec_json: serde_json::Value = serde_json::from_slice(&exec_body).expect("failure json");
    assert_eq!(
        exec_json.get("status").and_then(|v| v.as_str()),
        Some("failure")
    );
    assert_eq!(
        exec_json.get("message").and_then(|v| v.as_str()),
        Some("Access Denied")
    );
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
        .create_user(realm.id, "dana", "password-123", None, false)
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
        .create_user(realm.id, "erin", "password-123", None, false)
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
        .create_user(realm.id, "gina", "password-123", None, false)
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
        .create_user(realm.id, "frank", "password-123", None, false)
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
