-- Stores the high-level flows (e.g., "browser-login", "registration")
CREATE TABLE IF NOT EXISTS auth_flows
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);

-- Stores the ordered steps for each flow
CREATE TABLE IF NOT EXISTS auth_flow_steps
(
    id                 TEXT PRIMARY KEY NOT NULL, -- UUID
    flow_id            TEXT             NOT NULL,
    -- The "name" of the authenticator to run, e.g., "builtin-password-auth"
    authenticator_name TEXT             NOT NULL,
    -- The order to run this step in
    priority           INTEGER          NOT NULL,
    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE
);

-- Stores the state of an in-progress login
CREATE TABLE IF NOT EXISTS login_sessions
(
    id           TEXT PRIMARY KEY NOT NULL,           -- UUID for this specific login attempt
    realm_id     TEXT             NOT NULL,
    flow_id      TEXT             NOT NULL,
    current_step INTEGER          NOT NULL DEFAULT 0, -- The index of the step we are on
    user_id      TEXT,                                -- The user, once they are identified
    state_data   TEXT,                                -- JSON blob for authenticator state (e.g., "otp_secret")
    expires_at   TIMESTAMP        NOT NULL,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Stores admin configuration for each authenticator (e.g., OTP digits)
CREATE TABLE IF NOT EXISTS authenticator_config
(
    id                 TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id           TEXT             NOT NULL,
    -- The "name" of the authenticator this config is for
    authenticator_name TEXT             NOT NULL,
    -- The JSON blob of config data
    config_data        TEXT             NOT NULL,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, authenticator_name)
);