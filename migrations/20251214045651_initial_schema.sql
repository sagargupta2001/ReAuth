-- 1. REALMS
-- Core tenant table.
-- Note: flow_id columns are nullable and do not strictly enforce FKs here
-- to avoid circular dependencies during table creation (Realms <-> Flows).
CREATE TABLE IF NOT EXISTS realms
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
);

-- 2. USERS
-- Scoped to a Realm.
CREATE TABLE IF NOT EXISTS users
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT             NOT NULL,
    username        TEXT             NOT NULL,
    hashed_password TEXT             NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, username) -- Scoped uniqueness
);

-- 3. ROLES (Global/System level based on provided schema)
CREATE TABLE IF NOT EXISTS roles
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    name        TEXT             NOT NULL UNIQUE,
    description TEXT
);

-- 4. GROUPS (Global/System level based on provided schema)
CREATE TABLE IF NOT EXISTS groups
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    name        TEXT             NOT NULL UNIQUE,
    description TEXT
);

-- 5. PERMISSIONS & HIERARCHY
CREATE TABLE IF NOT EXISTS role_permissions
(
    role_id         TEXT NOT NULL,
    permission_name TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_name),
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS role_composite_roles
(
    parent_role_id TEXT NOT NULL,
    child_role_id  TEXT NOT NULL,
    PRIMARY KEY (parent_role_id, child_role_id),
    FOREIGN KEY (parent_role_id) REFERENCES roles (id) ON DELETE CASCADE,
    FOREIGN KEY (child_role_id) REFERENCES roles (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_groups
(
    user_id  TEXT NOT NULL,
    group_id TEXT NOT NULL,
    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS group_roles
(
    group_id TEXT NOT NULL,
    role_id  TEXT NOT NULL,
    PRIMARY KEY (group_id, role_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- 6. OIDC CLIENTS
CREATE TABLE IF NOT EXISTS oidc_clients
(
    id            TEXT PRIMARY KEY NOT NULL,
    realm_id      TEXT             NOT NULL,
    client_id     TEXT             NOT NULL UNIQUE,
    client_secret TEXT,
    redirect_uris TEXT             NOT NULL, -- JSON array
    scopes        TEXT             NOT NULL,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);

-- 7. AUTHENTICATION FLOWS & CONFIG
CREATE TABLE IF NOT EXISTS auth_flows
(
    id          TEXT PRIMARY KEY NOT NULL,                   -- UUID
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    alias       TEXT,
    type        TEXT             NOT NULL DEFAULT 'browser', -- 'browser', 'registration', 'direct'
    built_in    BOOLEAN          NOT NULL DEFAULT FALSE,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);

CREATE TABLE IF NOT EXISTS auth_flow_steps
(
    id                 TEXT PRIMARY KEY NOT NULL, -- UUID
    flow_id            TEXT             NOT NULL,
    authenticator_name TEXT             NOT NULL,
    priority           INTEGER          NOT NULL,
    requirement        TEXT             NOT NULL DEFAULT 'REQUIRED',
    config             TEXT,
    parent_step_id     TEXT,

    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS authenticator_config
(
    id                 TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id           TEXT             NOT NULL,
    authenticator_name TEXT             NOT NULL,
    config_data        TEXT             NOT NULL, -- JSON blob
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, authenticator_name)
);

-- 8. FLOW BUILDER (Drafts & Versions)
CREATE TABLE IF NOT EXISTS flow_drafts
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

CREATE TABLE IF NOT EXISTS flow_versions
(
    id                 TEXT PRIMARY KEY NOT NULL,
    flow_id            TEXT             NOT NULL,              -- Links to runtime auth_flows
    version_number     INTEGER          NOT NULL,
    execution_artifact TEXT             NOT NULL,              -- Compiled JSON
    checksum           TEXT             NOT NULL,
    graph_json         TEXT             NOT NULL DEFAULT '{}', -- For restoring UI state
    created_at         DATETIME         NOT NULL,

    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE,
    UNIQUE (flow_id, version_number)
);

CREATE TABLE IF NOT EXISTS flow_deployments
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

-- 9. RUNTIME STATE (Sessions & Tokens)
CREATE TABLE IF NOT EXISTS auth_sessions
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT             NOT NULL,
    flow_version_id TEXT             NOT NULL,
    current_node_id TEXT             NOT NULL,
    context         TEXT             NOT NULL DEFAULT '{}',
    status          TEXT             NOT NULL DEFAULT 'active',
    user_id         TEXT,
    created_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at      DATETIME         NOT NULL,

    FOREIGN KEY (realm_id) REFERENCES realms (id)
);
CREATE INDEX idx_auth_sessions_expires ON auth_sessions (expires_at);

CREATE TABLE IF NOT EXISTS authorization_codes
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

CREATE TABLE IF NOT EXISTS refresh_tokens
(
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT             NOT NULL,
    realm_id     TEXT             NOT NULL,
    client_id    TEXT,
    expires_at   DATETIME         NOT NULL,
    ip_address   TEXT,
    user_agent   TEXT,
    created_at   DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);