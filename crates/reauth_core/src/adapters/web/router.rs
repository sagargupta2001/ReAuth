use super::{
    auth_handler, auth_middleware, execution_handler, flow_handler, log_stream_handler,
    oidc_handler, plugin_handler, rbac_handler, realm_handler, server::ui_handler, session_handler,
    user_handler,
};
use crate::adapters::web::middleware::{cors_middleware, permission_guard};
use crate::AppState;
use crate::domain::permissions;
use axum::routing::{delete, get_service, put};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn create_router(app_state: AppState, plugins_path: PathBuf) -> Router {
    // 1. Public Routes
    let public_api = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/logs/ws", get(log_stream_handler::log_stream_handler))
        .route("/realms/{realm}/login", get(execution_handler::start_login))
        .nest("/execution", execution_routes())
        .nest("/realms/{realm}/auth", auth_routes())
        .nest("/realms/{realm}/oidc", oidc_routes())
        .nest("/plugins", plugin_routes())
        .nest("/realms/{realm}/users", public_user_routes());

    // 2. Protected Routes (Require Login)
    // We construct these using the corrected helper functions
    let protected_api = Router::new()
        .nest("/realms", realm_routes(app_state.clone()))
        .nest("/realms/{realm}/clients", client_routes(app_state.clone()))
        .nest("/realms/{realm}/rbac", rbac_routes(app_state.clone()))
        .nest(
            "/realms/{realm}/users",
            protected_user_routes(app_state.clone()),
        )
        .nest("/realms/{realm}/flows", flow_routes(app_state.clone()))
        // Apply Auth Guard to the ENTIRE protected block
        // This runs FIRST (Outer Layer), ensuring user is logged in before checking permissions.
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware::auth_guard,
        ));

    let api_router = Router::new().merge(public_api).merge(protected_api);

    Router::new()
        .nest("/api", api_router)
        .nest_service("/plugins", get_service(ServeDir::new(plugins_path)))
        .fallback(ui_handler::static_handler)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            cors_middleware::dynamic_cors_guard,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

// --- Corrected Route Definitions ---

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

// [FIXED] Split routes by permission requirement
fn protected_user_routes(state: AppState) -> Router<AppState> {
    // 1. No Special Permission (Just Auth)
    let base_routes = Router::new().route("/me", get(user_handler::get_me_handler));

    // 2. Read Permission
    let read_routes = Router::new()
        .route("/", get(user_handler::list_users_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::USER_READ)
            },
        ));

    // 3. Write Permission
    let write_routes = Router::new()
        .route("/{id}", put(user_handler::update_user_handler))
        .route("/{id}", get(user_handler::get_user_handler))
        .route("/{id}/roles", post(rbac_handler::assign_user_role_handler))
        .route(
            "/{id}/roles/{role_id}",
            delete(rbac_handler::remove_user_role_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::USER_WRITE)
            },
        ));

    // Merge them all
    base_routes.merge(read_routes).merge(write_routes)
}

// Split Realm Routes
fn realm_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route("/", get(realm_handler::list_realms_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_READ)
            },
        ));

    let write_routes = Router::new()
        .route("/", post(realm_handler::create_realm_handler))
        .route("/{id}", put(realm_handler::update_realm_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_WRITE)
            },
        ));

    // Session management needs USER_WRITE (or a specific permission)
    let session_routes = Router::new()
        .route(
            "/{realm}/sessions",
            get(session_handler::list_sessions_handler),
        )
        .route(
            "/{realm}/sessions/{id}",
            delete(session_handler::revoke_session_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::USER_WRITE)
            },
        ));

    read_routes.merge(write_routes).merge(session_routes)
}

fn rbac_routes(state: AppState) -> Router<AppState> {
    // All these require RBAC_WRITE, so we can keep them in one block
    // unless you want to separate Read (List Roles) from Write.
    Router::new()
        .route("/roles", post(rbac_handler::create_role_handler))
        .route("/roles", get(rbac_handler::list_roles_handler))
        .route(
            "/clients/{client_id}/roles",
            get(rbac_handler::list_client_roles_handler)
        )
        .route(
            "/roles/{id}/permissions",
            post(rbac_handler::assign_permission_handler)
                .get(rbac_handler::list_role_permissions_handler)
                .delete(rbac_handler::revoke_permission_handler)
        )
        .route(
            "/roles/{id}/members",
            get(rbac_handler::list_role_members_handler)
        )
        .route(
            "/roles/{id}/members/list",
            get(rbac_handler::list_role_members_page_handler)
        )
        .route(
            "/roles/{id}/permissions/bulk",
            post(rbac_handler::bulk_permissions_handler) // [NEW] Bulk
        )
        .route(
            "/groups",
            post(rbac_handler::create_group_handler).get(rbac_handler::list_groups_handler),
        )
        .route(
            "/groups/tree",
            get(rbac_handler::list_group_roots_handler),
        )
        .route(
            "/groups/{id}",
            get(rbac_handler::get_group_handler).put(rbac_handler::update_group_handler),
        )
        .route(
            "/groups/{id}/children",
            get(rbac_handler::list_group_children_handler),
        )
        .route(
            "/groups/{id}/move",
            post(rbac_handler::move_group_handler),
        )
        .route(
            "/groups/{id}/members",
            get(rbac_handler::list_group_members_handler)
                .post(rbac_handler::assign_user_to_group_handler),
        )
        .route(
            "/groups/{id}/members/list",
            get(rbac_handler::list_group_members_page_handler),
        )
        .route(
            "/groups/{id}/members/{user_id}",
            delete(rbac_handler::remove_user_from_group_handler),
        )
        .route(
            "/groups/{id}/roles",
            get(rbac_handler::list_group_roles_handler)
                .post(rbac_handler::assign_role_to_group_handler),
        )
        .route(
            "/groups/{id}/roles/list",
            get(rbac_handler::list_group_roles_page_handler),
        )
        .route(
            "/groups/{id}/roles/{role_id}",
            delete(rbac_handler::remove_role_from_group_handler),
        )
        .route(
            "/roles/{id}",
            delete(rbac_handler::delete_role_handler)
                .get(rbac_handler::get_role_handler)
                .put(rbac_handler::update_role_handler),
        )
        .route("/permissions", get(rbac_handler::list_permissions_handler))
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::RBAC_WRITE)
            },
        ))
}

fn client_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route("/", get(oidc_handler::list_clients_handler))
        .route("/{id}", get(oidc_handler::get_client_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::CLIENT_READ)
            },
        ));

    let write_routes = Router::new()
        .route("/", post(oidc_handler::create_client_handler))
        .route("/{id}", put(oidc_handler::update_client_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::CLIENT_CREATE)
            },
        ));

    read_routes.merge(write_routes)
}

fn flow_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(flow_handler::list_flows_handler))
        .route("/nodes", get(flow_handler::list_nodes_handler))
        .route("/drafts", get(flow_handler::list_drafts_handler))
        .route("/drafts", post(flow_handler::create_draft_handler))
        .route("/drafts/{id}", get(flow_handler::get_draft_handler))
        .route("/drafts/{id}", put(flow_handler::update_draft_handler))
        .route("/{id}/publish", post(flow_handler::publish_flow_handler))
        .route("/{id}/versions", get(flow_handler::list_versions_handler))
        .route("/{id}/rollback", post(flow_handler::rollback_flow_handler))
        .route(
            "/{id}/restore-draft",
            post(flow_handler::restore_draft_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_WRITE)
            },
        ))
}

fn execution_routes() -> Router<AppState> {
    Router::new().route("/{session_id}", post(execution_handler::submit_execution))
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
