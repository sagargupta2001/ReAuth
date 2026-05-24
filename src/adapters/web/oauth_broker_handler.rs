use crate::adapters::web::auth_handler::{
    create_clear_login_cookie, create_login_cookie, create_refresh_cookie,
};
use crate::constants::LOGIN_SESSION_COOKIE;
use crate::domain::auth_session::SessionStatus;
use crate::domain::execution::ExecutionResult;
use crate::domain::oidc::OidcContext;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use serde_json::json;
use url::Url;
use uuid::Uuid;

const CALLBACK_FAILURE_ACTION: &str = "idp_callback_failure";
const CALLBACK_INVALID_REQUEST_ACTION: &str = "idp_callback_invalid_request";
const CALLBACK_UPSTREAM_ERROR_ACTION: &str = "idp_callback_upstream_error";
const CALLBACK_SESSION_MISMATCH_ACTION: &str = "idp_callback_session_mismatch";

struct FailureRedirectContext {
    realm_id: Uuid,
    provider_id: Option<Uuid>,
    provider_alias: String,
    realm_name: String,
    message: String,
    session_id: Option<Uuid>,
    action: &'static str,
    extra_metadata: serde_json::Value,
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn oauth_start_handler(
    State(state): State<AppState>,
    Path((realm_name, alias)): Path<(String, String)>,
    jar: CookieJar,
    headers: HeaderMap,
    request: Request,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;
    let provider = state
        .identity_provider_service
        .get_domain_by_alias(realm.id, &alias)
        .await?;
    let session_id = resolve_target_session_id(&state, &jar, realm.id).await?;
    let ip_address = resolve_client_ip(&headers, &request);
    state
        .realm_idp_settings_service
        .enforce_oauth_start_rate_limit(realm.id, provider.id, &provider.alias, &ip_address)
        .await?;
    let redirect = state
        .oauth_broker_service
        .create_redirect(realm.id, &realm_name, session_id, &alias)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "redirect_url": redirect.redirect_url })),
    ))
}

fn resolve_client_ip(headers: &HeaderMap, request: &Request) -> String {
    if let Some(value) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    {
        if let Some(first_ip) = value.split(',').next() {
            let trimmed = first_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn oauth_callback_handler(
    State(state): State<AppState>,
    Path((realm_name, alias)): Path<(String, String)>,
    Query(query): Query<OAuthCallbackQuery>,
    jar: CookieJar,
) -> Result<Response> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;
    let provider = state
        .identity_provider_service
        .get_domain_by_alias(realm.id, &alias)
        .await
        .ok();

    state
        .audit_service
        .record(crate::domain::audit::NewAuditEvent {
            realm_id: realm.id,
            actor_user_id: None,
            action: "idp_callback_received".to_string(),
            target_type: "identity_provider".to_string(),
            target_id: provider.as_ref().map(|value| value.id.to_string()),
            metadata: json!({
                "provider_alias": alias,
                "state": query.state,
                "has_code": query.code.is_some(),
                "upstream_error": query.error,
            }),
        })
        .await?;

    if let Some(upstream_error) = query.error.as_deref() {
        let session_id = resolve_cookie_session_id(&jar)?;
        return failure_redirect(
            &state,
            FailureRedirectContext {
                realm_id: realm.id,
                provider_id: provider.as_ref().map(|value| value.id),
                provider_alias: alias.clone(),
                realm_name: realm_name.clone(),
                message: format!("Upstream provider returned '{}'.", upstream_error),
                session_id,
                action: CALLBACK_UPSTREAM_ERROR_ACTION,
                extra_metadata: json!({
                    "upstream_error": upstream_error
                }),
            },
        )
        .await;
    }

    let code = match query.code.as_deref() {
        Some(value) => value.to_string(),
        None => {
            let session_id = resolve_cookie_session_id(&jar)?;
            return failure_redirect(
                &state,
                FailureRedirectContext {
                    realm_id: realm.id,
                    provider_id: provider.as_ref().map(|value| value.id),
                    provider_alias: alias.clone(),
                    realm_name: realm_name.clone(),
                    message: "OAuth callback did not include an authorization code".to_string(),
                    session_id,
                    action: CALLBACK_INVALID_REQUEST_ACTION,
                    extra_metadata: json!({
                        "reason": "missing_code"
                    }),
                },
            )
            .await;
        }
    };
    let broker_state = match query.state.as_deref() {
        Some(value) => value.to_string(),
        None => {
            let session_id = resolve_cookie_session_id(&jar)?;
            return failure_redirect(
                &state,
                FailureRedirectContext {
                    realm_id: realm.id,
                    provider_id: provider.as_ref().map(|value| value.id),
                    provider_alias: alias.clone(),
                    realm_name: realm_name.clone(),
                    message: "OAuth callback did not include state".to_string(),
                    session_id,
                    action: CALLBACK_INVALID_REQUEST_ACTION,
                    extra_metadata: json!({
                        "reason": "missing_state"
                    }),
                },
            )
            .await;
        }
    };

    let callback = match state
        .oauth_broker_service
        .handle_callback(realm.id, &alias, &code, &broker_state)
        .await
    {
        Ok(value) => value,
        Err(err) => {
            let session_id = resolve_cookie_session_id(&jar)?;
            return failure_redirect(
                &state,
                FailureRedirectContext {
                    realm_id: realm.id,
                    provider_id: provider.as_ref().map(|value| value.id),
                    provider_alias: alias.clone(),
                    realm_name: realm_name.clone(),
                    message: err.to_string(),
                    session_id,
                    action: CALLBACK_FAILURE_ACTION,
                    extra_metadata: json!({}),
                },
            )
            .await;
        }
    };

    if resolve_cookie_session_id(&jar)? != Some(callback.auth_session_id) {
        return failure_redirect(
            &state,
            FailureRedirectContext {
                realm_id: realm.id,
                provider_id: provider.as_ref().map(|value| value.id),
                provider_alias: alias.clone(),
                realm_name: realm_name.clone(),
                message: "The OAuth login session was lost. Start the sign-in flow again."
                    .to_string(),
                session_id: None,
                action: CALLBACK_SESSION_MISMATCH_ACTION,
                extra_metadata: json!({
                    "expected_auth_session_id": callback.auth_session_id
                }),
            },
        )
        .await;
    }

    let mut session = state
        .auth_session_repo
        .find_by_id(&callback.auth_session_id)
        .await?
        .ok_or(Error::InvalidLoginSession)?;
    session.update_context(
        "oauth_broker_result",
        serde_json::to_value(&callback.broker_result)
            .map_err(|err| Error::System(format!("Failed to serialize broker result: {}", err)))?,
    );
    state.auth_session_repo.update(&session).await?;

    let execution = state
        .flow_executor
        .execute(
            callback.auth_session_id,
            Some(json!({ "oauth_callback": true })),
        )
        .await?;

    match execution {
        ExecutionResult::Success { redirect_url } => {
            finish_browser_flow_redirect(&state, callback.auth_session_id, redirect_url).await
        }
        ExecutionResult::Challenge { .. } | ExecutionResult::AwaitingAction { .. } => {
            redirect_back_to_login(&realm_name, Some(callback.auth_session_id), None)
        }
        ExecutionResult::Failure { reason } => {
            failure_redirect(
                &state,
                FailureRedirectContext {
                    realm_id: realm.id,
                    provider_id: provider.as_ref().map(|value| value.id),
                    provider_alias: alias.clone(),
                    realm_name: realm_name.clone(),
                    message: reason.clone(),
                    session_id: Some(callback.auth_session_id),
                    action: CALLBACK_FAILURE_ACTION,
                    extra_metadata: json!({}),
                },
            )
            .await
        }
        ExecutionResult::Continue => Err(Error::System(
            "OAuth callback reached an internal continue state".to_string(),
        )),
    }
}

async fn resolve_target_session_id(
    state: &AppState,
    jar: &CookieJar,
    realm_id: Uuid,
) -> Result<Uuid> {
    let cookies: Vec<_> = jar
        .iter()
        .filter(|cookie| cookie.name() == LOGIN_SESSION_COOKIE)
        .collect();
    for cookie in cookies {
        if let Ok(parse_id) = Uuid::parse_str(cookie.value()) {
            if let Ok(Some(session)) = state.auth_session_repo.find_by_id(&parse_id).await {
                if session.realm_id == realm_id && session.status == SessionStatus::Active {
                    return Ok(parse_id);
                }
            }
        }
    }
    Err(Error::InvalidLoginSession)
}

async fn finish_browser_flow_redirect(
    state: &AppState,
    session_id: Uuid,
    redirect_url: String,
) -> Result<Response> {
    let final_session = state
        .auth_session_repo
        .find_by_id(&session_id)
        .await?
        .ok_or(Error::InvalidLoginSession)?;
    let user_id = final_session
        .user_id
        .ok_or_else(|| Error::System("Authenticated user not found".to_string()))?;

    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&create_clear_login_cookie().to_string())?,
    );

    let target_url = if let Some(oidc_value) = final_session.context.get("oidc") {
        if let Ok(oidc_ctx) = serde_json::from_value::<OidcContext>(oidc_value.clone()) {
            if final_session.context.get("sso_token_id").is_none() {
                let user = state.user_service.get_user(user_id).await?;
                let (_, refresh_token) = state
                    .auth_service
                    .create_session(&user, None, None, None)
                    .await?;
                response_headers.append(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&create_refresh_cookie(&refresh_token).to_string())?,
                );
            }

            let auth_code = state
                .oidc_service
                .create_authorization_code(
                    final_session.realm_id,
                    user_id,
                    oidc_ctx.client_id,
                    oidc_ctx.redirect_uri.clone(),
                    oidc_ctx.nonce,
                    oidc_ctx.code_challenge,
                    oidc_ctx
                        .code_challenge_method
                        .unwrap_or_else(|| "S256".to_string()),
                )
                .await?;

            let mut url = Url::parse(&oidc_ctx.redirect_uri)
                .map_err(|_| Error::OidcInvalidRedirect(oidc_ctx.redirect_uri.clone()))?;
            url.query_pairs_mut().append_pair("code", &auth_code.code);
            if let Some(s) = oidc_ctx.state {
                url.query_pairs_mut().append_pair("state", &s);
            }
            url.to_string()
        } else {
            redirect_url
        }
    } else if redirect_url == "/" {
        let user = state.user_service.get_user(user_id).await?;
        let (_, refresh_token) = state
            .auth_service
            .create_session(&user, None, None, None)
            .await?;
        response_headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&create_refresh_cookie(&refresh_token).to_string())?,
        );
        redirect_url
    } else {
        redirect_url
    };

    response_headers.insert(header::LOCATION, HeaderValue::from_str(&target_url)?);
    Ok((StatusCode::FOUND, response_headers).into_response())
}

fn resolve_cookie_session_id(jar: &CookieJar) -> Result<Option<Uuid>> {
    let cookies: Vec<_> = jar
        .iter()
        .filter(|cookie| cookie.name() == LOGIN_SESSION_COOKIE)
        .collect();
    for cookie in cookies {
        if let Ok(session_id) = Uuid::parse_str(cookie.value()) {
            return Ok(Some(session_id));
        }
    }
    Ok(None)
}

async fn failure_redirect(state: &AppState, context: FailureRedirectContext) -> Result<Response> {
    let mut metadata = json!({
        "provider_alias": context.provider_alias.clone(),
        "message": context.message.clone(),
    });
    if let Some(metadata_obj) = metadata.as_object_mut() {
        if let Some(extra_obj) = context.extra_metadata.as_object() {
            for (key, value) in extra_obj {
                metadata_obj.insert(key.clone(), value.clone());
            }
        }
    }

    state
        .audit_service
        .record(crate::domain::audit::NewAuditEvent {
            realm_id: context.realm_id,
            actor_user_id: None,
            action: context.action.to_string(),
            target_type: "identity_provider".to_string(),
            target_id: context.provider_id.map(|value| value.to_string()),
            metadata,
        })
        .await?;

    if let Some(session_id) = context.session_id {
        if let Some(mut session) = state.auth_session_repo.find_by_id(&session_id).await? {
            session.update_context(
                "oauth_failure",
                json!({
                    "message": context.message,
                    "provider_alias": context.provider_alias.clone()
                }),
            );
            session.update_context(
                "oauth_selected_provider_alias",
                json!(context.provider_alias.clone()),
            );
            if let Some(map) = session.context.as_object_mut() {
                map.remove("oauth_broker_result");
                map.remove("oauth_link_error");
            }
            state.auth_session_repo.update(&session).await?;
        }
    }

    redirect_back_to_login(
        &context.realm_name,
        context.session_id,
        Some(&context.message),
    )
}

fn redirect_back_to_login(
    realm_name: &str,
    session_id: Option<Uuid>,
    error: Option<&str>,
) -> Result<Response> {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.append_pair("realm", realm_name);
    if session_id.is_none() {
        if let Some(message) = error {
            query.append_pair("oauth_error", message);
        }
    }
    let target = format!("/#/login?{}", query.finish());

    let mut headers = HeaderMap::new();
    let session_cookie = match session_id {
        Some(value) => create_login_cookie(value).to_string(),
        None => create_clear_login_cookie().to_string(),
    };
    headers.append(header::SET_COOKIE, HeaderValue::from_str(&session_cookie)?);
    headers.insert(header::LOCATION, HeaderValue::from_str(&target)?);
    Ok((StatusCode::FOUND, headers).into_response())
}
