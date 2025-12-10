use super::{
    auth_handler, auth_middleware, flow_handler, log_stream_handler, oidc_handler, plugin_handler,
    rbac_handler, realm_handler, server::ui_handler, session_handler, user_handler,
};
use crate::adapters::web::server::AppState;
use axum::routing::get_service;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE};
use http::Method;
use std::path::PathBuf;
use tower_http::cors::AllowOrigin;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

/// Creates the complete, state-filled Axum router for the application.
pub fn create_router(app_state: AppState, plugins_path: PathBuf) -> Router {
    // 1. Public routes that ANYONE can access.
    let public_api = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/logs/ws", get(log_stream_handler::log_stream_handler))
        .nest("/auth", auth_routes())
        .nest("/realms/{realm}/oidc", oidc_routes())
        .nest("/realms/{realm}/clients", client_routes())
        .nest("/plugins", plugin_routes())
        .nest("/realms/{realm}/users", public_user_routes());

    // 2. Protected routes that require the `auth_guard` middleware.
    let protected_api = Router::new()
        .nest("/realms/{realm}/users", protected_user_routes())
        .nest("/rbac", rbac_routes())
        .nest("/realms", realm_routes())
        .nest("/realms/{realm}/flows", flow_routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware::auth_guard,
        ));

    // Combine public and protected routes under the /api prefix
    let api_router = Router::new().merge(public_api).merge(protected_api);

    // --- Main Application Router ---
    let cors = CorsLayer::new()
        // Allow specific origin (Your Frontend URL)
        // This MUST match exactly. No trailing slash.
        .allow_origin(AllowOrigin::mirror_request())
        .allow_credentials(true)
        // Allow standard methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // Allow specific headers (Content-Type is needed for JSON)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, COOKIE, ACCEPT]);

    Router::new()
        .nest("/api", api_router)
        .nest_service("/plugins", get_service(ServeDir::new(plugins_path)))
        .fallback(ui_handler::static_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

// --- Route Definitions ---
// By breaking routes into small functions, this file stays clean and scalable.

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(auth_handler::start_login_flow_handler))
        .route(
            "/login/execute",
            post(auth_handler::execute_login_step_handler),
        )
        .route("/refresh", post(auth_handler::refresh_handler))
        .route("/logout", post(auth_handler::logout_handler))
}

fn public_user_routes() -> Router<AppState> {
    Router::new().route("/", post(user_handler::create_user_handler))
}

fn protected_user_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(user_handler::get_me_handler))
        .route("/", get(user_handler::list_users_handler))
        .route("/{id}", get(user_handler::get_user_handler))
        .route(
            "/{id}",
            axum::routing::put(user_handler::update_user_handler),
        )
}

fn realm_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(realm_handler::create_realm_handler))
        .route("/", get(realm_handler::list_realms_handler))
        .route(
            "/{id}",
            axum::routing::put(realm_handler::update_realm_handler),
        )
        .route(
            "/{realm}/sessions",
            get(session_handler::list_sessions_handler),
        )
        .route(
            "/{realm}/sessions/{id}",
            axum::routing::delete(session_handler::revoke_session_handler),
        )
}

fn rbac_routes() -> Router<AppState> {
    Router::new()
        .route("/roles", post(rbac_handler::create_role_handler))
        .route("/groups", post(rbac_handler::create_group_handler))
}

fn plugin_routes() -> Router<AppState> {
    Router::new()
        .route("/manifests", get(plugin_handler::get_plugin_manifests))
        .route("/{id}/say-hello", get(plugin_handler::plugin_proxy_handler))
        .route("/{id}/enable", post(plugin_handler::enable_plugin_handler))
        .route(
            "/{id}/disable",
            post(plugin_handler::disable_plugin_handler),
        )
}

fn oidc_routes() -> Router<AppState> {
    Router::new()
        .route("/authorize", get(oidc_handler::authorize_handler))
        .route("/token", post(oidc_handler::token_handler))
        .route("/.well-known/jwks.json", get(oidc_handler::jwks_handler))
}
fn client_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(oidc_handler::list_clients_handler))
        .route("/", post(oidc_handler::create_client_handler))
        .route("/{id}", get(oidc_handler::get_client_handler))
        .route(
            "/{id}",
            axum::routing::put(oidc_handler::update_client_handler),
        )
}

fn flow_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(flow_handler::list_flows_handler))
        // Node Registry
        .route("/nodes", get(flow_handler::list_nodes_handler))
        // Draft CRUD
        .route("/drafts", get(flow_handler::list_drafts_handler))
        .route("/drafts", post(flow_handler::create_draft_handler))
        .route("/drafts/{id}", get(flow_handler::get_draft_handler))
        .route(
            "/drafts/{id}",
            axum::routing::put(flow_handler::update_draft_handler),
        )
        .route("/{id}/publish", post(flow_handler::publish_flow_handler))
}
