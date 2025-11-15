use axum::routing::get_service;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use super::{
    auth_handler, auth_middleware, log_stream_handler, plugin_handler, rbac_handler, realm_handler,
    server::ui_handler, user_handler,
};
use crate::adapters::web::server::AppState;

/// Creates the complete, state-filled Axum router for the application.
pub fn create_router(app_state: AppState, plugins_path: PathBuf) -> Router {
    // --- Define Your API Routers ---

    // 1. Public routes that ANYONE can access.
    let public_api = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/logs/ws", get(log_stream_handler::log_stream_handler))
        .nest("/auth", auth_routes())
        .nest("/plugins", plugin_routes())
        .nest("/users", public_user_routes());

    // 2. Protected routes that require the `auth_guard` middleware.
    let protected_api = Router::new()
        .nest("/users", protected_user_routes())
        .nest("/rbac", rbac_routes())
        .nest("/realms", realm_routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware::auth_guard,
        ));

    // Combine public and protected routes under the /api prefix
    let api_router = Router::new().merge(public_api).merge(protected_api);

    // --- Main Application Router ---
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
    Router::new().route("/login", post(auth_handler::login_handler))
}

fn public_user_routes() -> Router<AppState> {
    Router::new().route("/", post(user_handler::create_user_handler))
}

fn protected_user_routes() -> Router<AppState> {
    Router::new().route("/me", get(user_handler::get_me_handler))
}

fn realm_routes() -> Router<AppState> {
    Router::new().route("/", post(realm_handler::create_realm_handler))
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
