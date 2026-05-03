use super::{
    audit_handler, auth_handler, auth_middleware, config_handler, execution_handler, flow_handler,
    harbor_handler, log_stream_handler, observability_handler, oidc_handler, rbac_handler,
    realm_email_handler, realm_handler, realm_passkey_handler, realm_recovery_handler,
    realm_security_headers_handler, search_handler, server::ui_handler, session_handler,
    setup_handler, theme_handler, user_handler, webhook_handler,
};
use crate::adapters::web::middleware::{
    cors_middleware, permission_guard, request_logging, security_headers,
};
use crate::domain::permissions;
use crate::AppState;
use axum::routing::{delete, put};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub fn create_router(app_state: AppState) -> Router {
    // 1. Public Routes
    let public_api = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/logs/ws", get(log_stream_handler::log_stream_handler))
        .route("/realms/{realm}/login", get(execution_handler::start_login))
        .nest("/execution", execution_routes())
        .nest("/realms/{realm}/auth", auth_routes())
        .nest("/realms/{realm}/oidc", oidc_routes())
        .nest("/realms/{realm}/theme", theme_routes())
        .nest("/realms/{realm}/users", public_user_routes());

    // 2. Protected Routes (Require Login)
    // We construct these using the corrected helper functions
    let protected_api = Router::new()
        .merge(config_routes(app_state.clone()))
        .nest("/realms", realm_routes(app_state.clone()))
        .nest("/realms/{realm}/clients", client_routes(app_state.clone()))
        .nest("/realms/{realm}/rbac", rbac_routes(app_state.clone()))
        .nest("/realms/{realm}/audits", audit_routes(app_state.clone()))
        .nest(
            "/realms/{realm}/users",
            protected_user_routes(app_state.clone()),
        )
        .nest("/realms/{realm}/flows", flow_routes(app_state.clone()))
        .nest("/realms/{realm}/harbor", harbor_routes(app_state.clone()))
        .nest(
            "/realms/{realm}/themes",
            theme_admin_routes(app_state.clone()),
        )
        .nest(
            "/realms/{realm}/webhooks",
            webhook_routes(app_state.clone()),
        )
        .route(
            "/realms/{realm}/search",
            get(search_handler::omni_search_handler),
        )
        // Apply Auth Guard to the ENTIRE protected block
        // This runs FIRST (Outer Layer), ensuring user is logged in before checking permissions.
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware::auth_guard,
        ));

    let api_router = Router::new()
        .merge(public_api)
        .merge(protected_api)
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            security_headers::attach_realm_security_headers,
        ))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            request_logging::log_api_request,
        ));

    let system_public_api = Router::new()
        .route("/setup/status", get(setup_handler::setup_status_handler))
        .route("/setup", post(setup_handler::setup_handler))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            request_logging::log_api_request,
        ));

    let system_api = Router::new()
        .nest("/observability", observability_routes(app_state.clone()))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware::auth_guard,
        ))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            request_logging::log_api_request,
        ));

    let system_router = system_public_api.merge(system_api);

    Router::new()
        .nest("/api", api_router)
        .nest("/api/system", system_router)
        .fallback(ui_handler::static_handler)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            cors_middleware::dynamic_cors_guard,
        ))
        .with_state(app_state)
}

// --- Corrected Route Definitions ---

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(auth_handler::start_login_flow_handler))
        .route(
            "/register",
            get(auth_handler::start_registration_flow_handler),
        )
        .route("/reset", get(auth_handler::start_reset_flow_handler))
        .route(
            "/passkeys/authenticate/options",
            post(auth_handler::passkey_authenticate_options_handler),
        )
        .route(
            "/passkeys/authenticate/verify",
            post(auth_handler::passkey_authenticate_verify_handler),
        )
        .route(
            "/passkeys/enroll/options",
            post(auth_handler::passkey_enroll_options_handler),
        )
        .route(
            "/passkeys/enroll/verify",
            post(auth_handler::passkey_enroll_verify_handler),
        )
        .route(
            "/login/execute",
            post(auth_handler::execute_login_step_handler),
        )
        .route(
            "/register/execute",
            post(auth_handler::execute_login_step_handler),
        )
        .route(
            "/reset/execute",
            post(auth_handler::execute_reset_step_handler),
        )
        .route("/resume", post(auth_handler::resume_action_handler))
        .route("/resend", post(auth_handler::resend_action_handler))
        .route("/action-status", get(auth_handler::action_status_handler))
        .route("/refresh", post(auth_handler::refresh_handler))
        .route("/logout", post(auth_handler::logout_handler))
}

fn config_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/config/reload",
            post(config_handler::reload_config_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_WRITE)
            },
        ))
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
        .route("/", delete(user_handler::delete_users_handler))
        .route("/{id}", put(user_handler::update_user_handler))
        .route("/{id}", get(user_handler::get_user_handler))
        .route(
            "/{id}/credentials",
            get(user_handler::list_user_credentials_handler),
        )
        .route(
            "/{id}/credentials/password",
            put(user_handler::update_user_password_handler),
        )
        .route(
            "/{id}/credentials/passkeys/{credential_id}",
            delete(user_handler::revoke_user_passkey_handler),
        )
        .route(
            "/{id}/credentials/passkeys/{credential_id}",
            put(user_handler::update_user_passkey_metadata_handler),
        )
        .route(
            "/{id}/credentials/password-policy",
            put(user_handler::update_user_password_policy_handler),
        )
        .route(
            "/{id}/roles",
            get(rbac_handler::list_user_roles_handler).post(rbac_handler::assign_user_role_handler),
        )
        .route(
            "/{id}/roles/list",
            get(rbac_handler::list_user_roles_page_handler),
        )
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
        .route(
            "/{id}/email-settings",
            get(realm_email_handler::get_realm_email_settings_handler),
        )
        .route(
            "/{id}/recovery-settings",
            get(realm_recovery_handler::get_realm_recovery_settings_handler),
        )
        .route(
            "/{id}/passkey-settings",
            get(realm_passkey_handler::get_realm_passkey_settings_handler),
        )
        .route(
            "/{id}/passkey-settings/analytics",
            get(realm_passkey_handler::get_realm_passkey_analytics_handler),
        )
        .route(
            "/{id}/security-headers",
            get(realm_security_headers_handler::get_realm_security_headers_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_READ)
            },
        ));

    let write_routes = Router::new()
        .route("/", post(realm_handler::create_realm_handler))
        .route(
            "/bootstrap/import",
            post(harbor_handler::bootstrap_import_harbor_bundle_handler),
        )
        .route(
            "/bootstrap/import/archive",
            post(harbor_handler::bootstrap_import_harbor_archive_handler),
        )
        .route("/{id}", put(realm_handler::update_realm_handler))
        .route(
            "/{id}/email-settings",
            put(realm_email_handler::update_realm_email_settings_handler),
        )
        .route(
            "/{id}/email-settings/test",
            post(realm_email_handler::test_realm_email_settings_handler),
        )
        .route(
            "/{id}/recovery-settings",
            put(realm_recovery_handler::update_realm_recovery_settings_handler),
        )
        .route(
            "/{id}/passkey-settings",
            put(realm_passkey_handler::update_realm_passkey_settings_handler),
        )
        .route(
            "/{id}/passkey-settings/recommended-browser-flow",
            post(realm_passkey_handler::apply_recommended_passkey_browser_flow_handler),
        )
        .route(
            "/{id}/passkey-settings/recommended-registration-flow",
            post(realm_passkey_handler::apply_recommended_passkey_registration_flow_handler),
        )
        .route(
            "/{id}/security-headers",
            put(realm_security_headers_handler::update_realm_security_headers_handler),
        )
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
            get(rbac_handler::list_client_roles_handler),
        )
        .route(
            "/roles/{id}/permissions",
            post(rbac_handler::assign_permission_handler)
                .get(rbac_handler::list_role_permissions_handler)
                .delete(rbac_handler::revoke_permission_handler),
        )
        .route(
            "/roles/{id}/composites",
            get(rbac_handler::list_role_composites_handler)
                .post(rbac_handler::assign_composite_role_handler),
        )
        .route(
            "/roles/{id}/composites/list",
            get(rbac_handler::list_role_composites_page_handler),
        )
        .route(
            "/roles/{id}/composites/{child_role_id}",
            delete(rbac_handler::remove_composite_role_handler),
        )
        .route(
            "/roles/{id}/members",
            get(rbac_handler::list_role_members_handler),
        )
        .route(
            "/roles/{id}/members/list",
            get(rbac_handler::list_role_members_page_handler),
        )
        .route(
            "/roles/{id}/permissions/bulk",
            post(rbac_handler::bulk_permissions_handler), // [NEW] Bulk
        )
        .route(
            "/groups",
            post(rbac_handler::create_group_handler).get(rbac_handler::list_groups_handler),
        )
        .route("/groups/tree", get(rbac_handler::list_group_roots_handler))
        .route(
            "/groups/{id}",
            get(rbac_handler::get_group_handler)
                .put(rbac_handler::update_group_handler)
                .delete(rbac_handler::delete_group_handler),
        )
        .route(
            "/groups/{id}/delete-summary",
            get(rbac_handler::get_group_delete_summary_handler),
        )
        .route(
            "/groups/{id}/children",
            get(rbac_handler::list_group_children_handler),
        )
        .route("/groups/{id}/move", post(rbac_handler::move_group_handler))
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
        .route(
            "/permissions/custom",
            post(rbac_handler::create_custom_permission_handler),
        )
        .route(
            "/permissions/custom/{id}",
            put(rbac_handler::update_custom_permission_handler)
                .delete(rbac_handler::delete_custom_permission_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::RBAC_WRITE)
            },
        ))
}

fn webhook_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(webhook_handler::list_webhooks_handler))
        .route("/", post(webhook_handler::create_webhook_handler))
        .route(
            "/metrics",
            get(webhook_handler::event_routing_metrics_handler),
        )
        .route("/{id}", get(webhook_handler::get_webhook_handler))
        .route("/{id}", put(webhook_handler::update_webhook_handler))
        .route("/{id}", delete(webhook_handler::delete_webhook_handler))
        .route(
            "/{id}/enable",
            post(webhook_handler::enable_webhook_handler),
        )
        .route(
            "/{id}/disable",
            post(webhook_handler::disable_webhook_handler),
        )
        .route(
            "/{id}/roll-secret",
            post(webhook_handler::roll_webhook_secret_handler),
        )
        .route(
            "/{id}/subscriptions",
            post(webhook_handler::update_webhook_subscriptions_handler),
        )
        .route("/{id}/test", post(webhook_handler::test_webhook_handler))
        .route(
            "/{id}/deliveries",
            get(webhook_handler::list_webhook_deliveries_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_WRITE)
            },
        ))
}

fn theme_routes() -> Router<AppState> {
    Router::new()
        .route("/resolve", get(theme_handler::resolve_theme_handler))
        .route(
            "/{theme_id}/assets/{asset_id}",
            get(theme_handler::get_theme_asset_handler),
        )
}

fn theme_admin_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(theme_handler::list_themes_handler).post(theme_handler::create_theme_handler),
        )
        .route("/active", get(theme_handler::get_active_theme_handler))
        .route("/pages", get(theme_handler::list_theme_pages_handler))
        .route(
            "/{theme_id}",
            get(theme_handler::get_theme_handler).put(theme_handler::update_theme_handler),
        )
        .route(
            "/{theme_id}/preview",
            get(theme_handler::preview_theme_handler),
        )
        .route(
            "/{theme_id}/template-gaps",
            get(theme_handler::list_theme_template_gaps_handler),
        )
        .route(
            "/{theme_id}/bindings",
            get(theme_handler::list_theme_bindings_handler),
        )
        .route(
            "/client-bindings/{client_id}",
            get(theme_handler::get_theme_binding_handler),
        )
        .route(
            "/{theme_id}/bindings/{client_id}",
            put(theme_handler::upsert_theme_binding_handler)
                .delete(theme_handler::delete_theme_binding_handler),
        )
        .route(
            "/{theme_id}/draft",
            get(theme_handler::get_theme_draft_handler)
                .put(theme_handler::save_theme_draft_handler),
        )
        .route(
            "/{theme_id}/assets",
            get(theme_handler::list_theme_assets_handler)
                .post(theme_handler::upload_theme_asset_handler),
        )
        .route(
            "/{theme_id}/versions",
            get(theme_handler::list_theme_versions_handler),
        )
        .route(
            "/{theme_id}/publish",
            post(theme_handler::publish_theme_handler),
        )
        .route(
            "/{theme_id}/versions/{version_id}/activate",
            post(theme_handler::activate_theme_version_handler),
        )
        .route(
            "/{theme_id}/versions/{version_id}/draft",
            post(theme_handler::start_theme_draft_from_version_handler),
        )
        .route(
            "/{theme_id}/versions/{version_id}/snapshot",
            get(theme_handler::get_theme_version_snapshot_handler),
        )
        .route(
            "/{theme_id}/export",
            get(theme_handler::export_theme_bundle_handler),
        )
        .route(
            "/{theme_id}/import",
            post(theme_handler::import_theme_bundle_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::REALM_WRITE)
            },
        ))
}

fn audit_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(audit_handler::list_audit_events_handler))
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::EVENT_READ)
            },
        ))
}

fn observability_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/logs", get(observability_handler::list_logs_handler))
        .route(
            "/logs/targets",
            get(observability_handler::list_log_targets_handler),
        )
        .route(
            "/logs/clear",
            post(observability_handler::clear_logs_handler),
        )
        .route(
            "/deliveries",
            get(observability_handler::list_delivery_logs_handler),
        )
        .route(
            "/deliveries/{delivery_id}/replay",
            post(observability_handler::replay_delivery_handler),
        )
        .route("/traces", get(observability_handler::list_traces_handler))
        .route(
            "/traces/clear",
            post(observability_handler::clear_traces_handler),
        )
        .route(
            "/traces/{trace_id}",
            get(observability_handler::list_trace_spans_handler),
        )
        .route("/metrics", get(observability_handler::metrics_handler))
        .route(
            "/cache/stats",
            get(observability_handler::cache_stats_handler),
        )
        .route(
            "/cache/flush",
            post(observability_handler::cache_flush_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::EVENT_READ)
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

    let create_routes = Router::new()
        .route("/", post(oidc_handler::create_client_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::CLIENT_CREATE)
            },
        ));

    let update_routes = Router::new()
        .route("/{id}", put(oidc_handler::update_client_handler))
        .route(
            "/{id}/rotate-secret",
            post(oidc_handler::rotate_client_secret_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state,
            move |state, req, next| {
                permission_guard::require_permission(state, req, next, permissions::CLIENT_UPDATE)
            },
        ));

    read_routes.merge(create_routes).merge(update_routes)
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

fn harbor_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/export",
            post(harbor_handler::export_harbor_bundle_handler),
        )
        .route(
            "/export/archive",
            post(harbor_handler::export_harbor_archive_handler),
        )
        .route(
            "/import",
            post(harbor_handler::import_harbor_bundle_handler),
        )
        .route(
            "/import/archive",
            post(harbor_handler::import_harbor_archive_handler),
        )
        .route("/jobs", get(harbor_handler::list_harbor_jobs_handler))
        .route(
            "/jobs/{job_id}/conflicts",
            get(harbor_handler::list_harbor_job_conflicts_handler),
        )
        .route(
            "/jobs/{job_id}",
            get(harbor_handler::get_harbor_job_handler),
        )
        .route(
            "/jobs/{job_id}/details",
            get(harbor_handler::get_harbor_job_details_handler),
        )
        .route(
            "/jobs/{job_id}/download",
            get(harbor_handler::download_harbor_job_handler),
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

fn oidc_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(oidc_handler::discovery_handler),
        )
        .route("/authorize", get(oidc_handler::authorize_handler))
        .route("/token", post(oidc_handler::token_handler))
        .route("/userinfo", get(oidc_handler::userinfo_handler))
        .route("/.well-known/jwks.json", get(oidc_handler::jwks_handler))
}
