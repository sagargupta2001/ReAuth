-- Consolidated initial schema.

-- Generated from the current end-state of all historical local migrations.

-- Safe to use only for fresh local databases during early development.

CREATE TABLE realms
(
    id                        TEXT PRIMARY KEY NOT NULL,                -- UUID
    name                      TEXT             NOT NULL UNIQUE,
    access_token_ttl_secs     INTEGER          NOT NULL DEFAULT 900,    -- 15 minutes
    refresh_token_ttl_secs    INTEGER          NOT NULL DEFAULT 604800, -- 7 days

    -- Default Flows
    browser_flow_id           TEXT,
    registration_flow_id      TEXT,
    direct_grant_flow_id      TEXT,
    reset_credentials_flow_id TEXT
, pkce_required_public_clients INTEGER NOT NULL DEFAULT 1, lockout_threshold INTEGER NOT NULL DEFAULT 5, lockout_duration_secs INTEGER NOT NULL DEFAULT 900, is_system INTEGER NOT NULL DEFAULT 0, registration_enabled INTEGER NOT NULL DEFAULT 1, default_registration_role_ids TEXT NOT NULL DEFAULT '[]');
CREATE TABLE users
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT             NOT NULL,
    username        TEXT             NOT NULL,
    hashed_password TEXT             NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP, email TEXT,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, username) -- Scoped uniqueness
);
CREATE TABLE roles
(
    id          TEXT PRIMARY KEY NOT NULL,
    realm_id    TEXT             NOT NULL, -- Always belongs to a realm
    client_id   TEXT,                      -- NULL = Realm Role, NOT NULL = Client Role
    name        TEXT             NOT NULL,
    description TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oidc_clients (id) ON DELETE CASCADE,

    -- Constraint: A role name must be unique within its "namespace" (Realm or Client)
    UNIQUE (realm_id, client_id, name)
);
CREATE TABLE groups
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP, parent_id TEXT REFERENCES groups (id) ON DELETE SET NULL, sort_order INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);
CREATE TABLE role_permissions
(
    role_id         TEXT NOT NULL,
    permission_name TEXT NOT NULL, -- e.g., "client:create", "realm:write"

    PRIMARY KEY (role_id, permission_name),
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);
CREATE TABLE role_composite_roles
(
    parent_role_id TEXT NOT NULL,
    child_role_id  TEXT NOT NULL,

    PRIMARY KEY (parent_role_id, child_role_id),
    FOREIGN KEY (parent_role_id) REFERENCES roles (id) ON DELETE CASCADE,
    FOREIGN KEY (child_role_id) REFERENCES roles (id) ON DELETE CASCADE,
    CHECK (parent_role_id <> child_role_id) -- Prevent self-reference
);
CREATE TABLE user_groups
(
    user_id  TEXT NOT NULL,
    group_id TEXT NOT NULL,

    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE
);
CREATE TABLE group_roles
(
    group_id TEXT NOT NULL,
    role_id  TEXT NOT NULL,

    PRIMARY KEY (group_id, role_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);
CREATE TABLE user_roles
(
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,

    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);
CREATE INDEX idx_roles_realm ON roles (realm_id);
CREATE INDEX idx_groups_realm ON groups (realm_id);
CREATE INDEX idx_user_groups_user ON user_groups (user_id);
CREATE INDEX idx_group_roles_group ON group_roles (group_id);
CREATE INDEX idx_user_roles_user ON user_roles (user_id);
CREATE INDEX idx_role_composite_parent ON role_composite_roles (parent_role_id);
CREATE INDEX idx_role_composite_child ON role_composite_roles (child_role_id);
CREATE TABLE auth_flows
(
    id          TEXT PRIMARY KEY NOT NULL,
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    alias       TEXT,
    type        TEXT             NOT NULL DEFAULT 'browser',
    built_in    BOOLEAN          NOT NULL DEFAULT FALSE,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);
CREATE TABLE flow_drafts
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    graph_json  TEXT             NOT NULL, -- Raw React Flow JSON
    flow_type   TEXT             NOT NULL DEFAULT 'browser',
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);
CREATE TABLE flow_versions
(
    id                 TEXT PRIMARY KEY NOT NULL,
    flow_id            TEXT             NOT NULL,              -- Links to runtime auth_flows
    version_number     INTEGER          NOT NULL,
    execution_artifact TEXT             NOT NULL,              -- Compiled JSON
    checksum           TEXT             NOT NULL,
    graph_json         TEXT             NOT NULL DEFAULT '{}', -- For restoring UI state
    created_at         DATETIME         NOT NULL, node_contract_versions TEXT NOT NULL DEFAULT '{}',

    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE,
    UNIQUE (flow_id, version_number)
);
CREATE TABLE flow_deployments
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    flow_type         TEXT             NOT NULL,
    active_version_id TEXT             NOT NULL,
    updated_at        DATETIME         NOT NULL,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (active_version_id) REFERENCES flow_versions (id) ON DELETE CASCADE,
    UNIQUE (realm_id, flow_type)
);
CREATE TABLE auth_sessions
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT             NOT NULL,
    flow_version_id TEXT             NOT NULL,
    current_node_id TEXT             NOT NULL,
    context         TEXT             NOT NULL DEFAULT '{}',
    execution_state TEXT             NOT NULL DEFAULT 'idle', -- idle, waiting_for_input, waiting_for_async
    last_ui_output  TEXT,                                     -- The last JSON form schema sent to the UI
    status          TEXT             NOT NULL DEFAULT 'active',
    user_id         TEXT,
    created_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at      DATETIME         NOT NULL,

    FOREIGN KEY (realm_id) REFERENCES realms (id)
);
CREATE INDEX idx_auth_sessions_expires ON auth_sessions (expires_at);
CREATE TABLE authorization_codes
(
    code                  TEXT PRIMARY KEY NOT NULL,
    user_id               TEXT             NOT NULL,
    client_id             TEXT             NOT NULL,
    redirect_uri          TEXT             NOT NULL,
    nonce                 TEXT,
    code_challenge        TEXT,
    code_challenge_method TEXT             NOT NULL,
    expires_at            DATETIME         NOT NULL
);
CREATE TABLE refresh_tokens
(
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT             NOT NULL,
    realm_id     TEXT             NOT NULL,
    client_id    TEXT,
    expires_at   DATETIME         NOT NULL,
    ip_address   TEXT,
    user_agent   TEXT,
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP, family_id TEXT, revoked_at DATETIME, replaced_by TEXT,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);
CREATE INDEX idx_groups_parent ON groups (parent_id);
CREATE INDEX idx_groups_parent_sort ON groups (parent_id, sort_order);
CREATE TABLE custom_permissions
(
    id          TEXT NOT NULL PRIMARY KEY,
    realm_id    TEXT NOT NULL,
    client_id   TEXT NULL,
    permission  TEXT NOT NULL,
    name        TEXT NOT NULL,
    description TEXT NULL,
    created_by  TEXT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oidc_clients (id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE SET NULL,
    UNIQUE (realm_id, client_id, permission)
);
CREATE INDEX idx_custom_permissions_realm ON custom_permissions (realm_id);
CREATE INDEX idx_custom_permissions_client ON custom_permissions (client_id);
CREATE INDEX idx_custom_permissions_permission ON custom_permissions (permission);
CREATE TABLE seed_history (
  name TEXT PRIMARY KEY,
  version INTEGER NOT NULL,
  checksum TEXT NOT NULL,
  applied_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP)
);
CREATE TABLE audit_events
(
    id             TEXT PRIMARY KEY NOT NULL,
    realm_id       TEXT             NOT NULL,
    actor_user_id  TEXT,
    action         TEXT             NOT NULL,
    target_type    TEXT             NOT NULL,
    target_id      TEXT,
    metadata       TEXT             NOT NULL DEFAULT '{}',
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (actor_user_id) REFERENCES users (id) ON DELETE SET NULL
);
CREATE INDEX idx_audit_events_realm_created_at
    ON audit_events (realm_id, created_at);
CREATE INDEX idx_audit_events_actor
    ON audit_events (actor_user_id);
CREATE INDEX idx_audit_events_action
    ON audit_events (action);
CREATE TABLE webhook_endpoints
(
    id                   TEXT PRIMARY KEY NOT NULL,
    realm_id             TEXT             NOT NULL,
    name                 TEXT             NOT NULL,
    url                  TEXT             NOT NULL,
    status               TEXT             NOT NULL DEFAULT 'active',
    signing_secret       TEXT             NOT NULL,
    custom_headers       TEXT             NOT NULL DEFAULT '{}',
    description          TEXT,
    consecutive_failures INTEGER          NOT NULL DEFAULT 0,
    last_failure_at      DATETIME,
    disabled_at          DATETIME,
    disabled_reason      TEXT,
    created_at           DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP, http_method TEXT NOT NULL DEFAULT 'POST', last_fired_at DATETIME,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);
CREATE INDEX idx_webhook_endpoints_realm
    ON webhook_endpoints (realm_id);
CREATE INDEX idx_webhook_endpoints_status
    ON webhook_endpoints (status);
CREATE TABLE webhook_subscriptions
(
    endpoint_id TEXT    NOT NULL,
    event_type  TEXT    NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (endpoint_id, event_type),
    FOREIGN KEY (endpoint_id) REFERENCES webhook_endpoints (id) ON DELETE CASCADE
);
CREATE INDEX idx_webhook_subscriptions_event_type
    ON webhook_subscriptions (event_type);
CREATE TABLE event_outbox
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT,
    event_type      TEXT             NOT NULL,
    event_version   TEXT             NOT NULL DEFAULT 'v1',
    occurred_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    payload_json    TEXT             NOT NULL,
    status          TEXT             NOT NULL DEFAULT 'pending',
    attempt_count   INTEGER          NOT NULL DEFAULT 0,
    next_attempt_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    locked_at       DATETIME,
    locked_by       TEXT,
    last_error      TEXT,
    created_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);
CREATE INDEX idx_event_outbox_status_next_attempt
    ON event_outbox (status, next_attempt_at);
CREATE INDEX idx_event_outbox_realm_occurred_at
    ON event_outbox (realm_id, occurred_at);
CREATE INDEX idx_event_outbox_event_type
    ON event_outbox (event_type);
CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens (family_id);
CREATE INDEX idx_refresh_tokens_revoked_at ON refresh_tokens (revoked_at);
CREATE INDEX idx_refresh_tokens_replaced_by ON refresh_tokens (replaced_by);
CREATE TABLE login_attempts
(
    realm_id       TEXT     NOT NULL,
    username       TEXT     NOT NULL,
    failed_count   INTEGER  NOT NULL DEFAULT 0,
    locked_until   DATETIME,
    last_failed_at DATETIME,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (realm_id, username),
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);
CREATE INDEX idx_login_attempts_realm
    ON login_attempts (realm_id);
CREATE INDEX idx_login_attempts_locked_until
    ON login_attempts (locked_until);
CREATE TABLE auth_session_actions
(
    id             TEXT PRIMARY KEY NOT NULL,
    session_id     TEXT             NOT NULL,
    realm_id       TEXT             NOT NULL,
    action_type    TEXT             NOT NULL,
    token_hash     TEXT             NOT NULL,
    payload_json   TEXT,
    resume_node_id TEXT,
    expires_at     DATETIME         NOT NULL,
    consumed_at    DATETIME,
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (session_id) REFERENCES auth_sessions (id) ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (token_hash)
);
CREATE INDEX idx_auth_session_actions_session ON auth_session_actions (session_id);
CREATE INDEX idx_auth_session_actions_expires ON auth_session_actions (expires_at);
CREATE INDEX idx_auth_session_actions_token ON auth_session_actions (token_hash);
CREATE TABLE themes
(
    id          TEXT PRIMARY KEY NOT NULL,
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP, is_system INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);
CREATE INDEX idx_themes_realm
    ON themes (realm_id);
CREATE TABLE theme_tokens
(
    id          TEXT PRIMARY KEY NOT NULL,
    theme_id    TEXT             NOT NULL,
    tokens_json TEXT             NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id)
);
CREATE INDEX idx_theme_tokens_theme
    ON theme_tokens (theme_id);
CREATE TABLE theme_layouts
(
    id          TEXT PRIMARY KEY NOT NULL,
    theme_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    layout_json TEXT             NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, name)
);
CREATE INDEX idx_theme_layouts_theme
    ON theme_layouts (theme_id);
CREATE TABLE theme_nodes
(
    id             TEXT PRIMARY KEY NOT NULL,
    theme_id       TEXT             NOT NULL,
    node_key       TEXT             NOT NULL,
    blueprint_json TEXT             NOT NULL,
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, node_key)
);
CREATE INDEX idx_theme_nodes_theme
    ON theme_nodes (theme_id);
CREATE TABLE theme_assets
(
    id         TEXT PRIMARY KEY NOT NULL,
    theme_id   TEXT             NOT NULL,
    asset_type TEXT             NOT NULL,
    filename   TEXT             NOT NULL,
    mime_type  TEXT             NOT NULL,
    byte_size  INTEGER          NOT NULL,
    checksum   TEXT,
    data       BLOB             NOT NULL,
    created_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE
);
CREATE INDEX idx_theme_assets_theme
    ON theme_assets (theme_id);
CREATE TABLE theme_versions
(
    id             TEXT PRIMARY KEY NOT NULL,
    theme_id       TEXT             NOT NULL,
    version_number INTEGER          NOT NULL,
    status         TEXT             NOT NULL DEFAULT 'draft',
    snapshot_json  TEXT             NOT NULL,
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, version_number)
);
CREATE INDEX idx_theme_versions_theme
    ON theme_versions (theme_id);
CREATE TABLE theme_bindings
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    client_id         TEXT,
    theme_id          TEXT             NOT NULL,
    active_version_id TEXT             NOT NULL,
    created_at        DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    FOREIGN KEY (active_version_id) REFERENCES theme_versions (id) ON DELETE CASCADE,
    UNIQUE (realm_id, client_id)
);
CREATE INDEX idx_theme_bindings_realm
    ON theme_bindings (realm_id);
CREATE INDEX idx_themes_system
    ON themes (is_system);
CREATE TABLE theme_draft_meta (
    theme_id TEXT PRIMARY KEY,
    draft_exists INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (theme_id) REFERENCES themes(id) ON DELETE CASCADE
);
CREATE TABLE harbor_jobs (
    id TEXT PRIMARY KEY,
    realm_id TEXT NOT NULL,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    scope TEXT NOT NULL,
    total_resources INTEGER NOT NULL DEFAULT 0,
    processed_resources INTEGER NOT NULL DEFAULT 0,
    created_count INTEGER NOT NULL DEFAULT 0,
    updated_count INTEGER NOT NULL DEFAULT 0,
    dry_run INTEGER NOT NULL DEFAULT 0,
    conflict_policy TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT
, artifact_path TEXT, artifact_filename TEXT, artifact_content_type TEXT);
CREATE INDEX idx_harbor_jobs_realm_created_at
    ON harbor_jobs (realm_id, created_at DESC);
CREATE TABLE harbor_job_conflicts (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    resource_key TEXT NOT NULL,
    action TEXT NOT NULL,
    policy TEXT NOT NULL,
    original_id TEXT,
    resolved_id TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_harbor_job_conflicts_job
    ON harbor_job_conflicts (job_id, created_at);
CREATE TABLE IF NOT EXISTS "oidc_clients"
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    client_id         TEXT             NOT NULL,
    client_secret     TEXT,
    redirect_uris     TEXT             NOT NULL,
    scopes            TEXT             NOT NULL,
    web_origins       TEXT             NOT NULL DEFAULT '[]',
    managed_by_config BOOLEAN          NOT NULL DEFAULT FALSE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, client_id)
);
CREATE TABLE realm_email_settings (
    realm_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    from_address TEXT,
    from_name TEXT,
    reply_to_address TEXT,
    smtp_host TEXT,
    smtp_port INTEGER,
    smtp_username TEXT,
    smtp_password TEXT,
    smtp_security TEXT NOT NULL DEFAULT 'starttls',
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);
CREATE INDEX idx_realm_email_settings_realm_id
    ON realm_email_settings(realm_id);
CREATE TABLE recovery_attempts (
    realm_id TEXT NOT NULL,
    identifier TEXT NOT NULL,
    window_started_at DATETIME NOT NULL,
    attempt_count INTEGER NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (realm_id, identifier),
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);
CREATE INDEX idx_recovery_attempts_realm
    ON recovery_attempts(realm_id);
CREATE TABLE realm_recovery_settings (
    realm_id TEXT PRIMARY KEY,
    token_ttl_minutes INTEGER NOT NULL DEFAULT 15,
    rate_limit_max INTEGER NOT NULL DEFAULT 5,
    rate_limit_window_minutes INTEGER NOT NULL DEFAULT 15,
    revoke_sessions_on_reset INTEGER NOT NULL DEFAULT 1,
    email_subject TEXT,
    email_body TEXT,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);
CREATE INDEX idx_realm_recovery_settings_realm_id
    ON realm_recovery_settings(realm_id);
CREATE TABLE realm_security_headers (
    realm_id TEXT PRIMARY KEY,
    x_frame_options TEXT,
    content_security_policy TEXT,
    x_content_type_options TEXT,
    referrer_policy TEXT,
    strict_transport_security TEXT,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);
CREATE INDEX idx_realm_security_headers_realm_id
    ON realm_security_headers(realm_id);
CREATE UNIQUE INDEX users_realm_email_unique ON users (realm_id, email);
