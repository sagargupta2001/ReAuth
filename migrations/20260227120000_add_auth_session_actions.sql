CREATE TABLE IF NOT EXISTS auth_session_actions
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
