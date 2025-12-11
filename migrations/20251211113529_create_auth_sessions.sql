CREATE TABLE auth_sessions
(
    id                        TEXT PRIMARY KEY NOT NULL,
    realm_id                  TEXT             NOT NULL,
    flow_version_id           TEXT             NOT NULL,

    -- Graph Pointer
    current_node_id           TEXT             NOT NULL,

    -- Legacy field (You can keep it if migration is hard, otherwise remove it)
    current_instruction_index INTEGER          NOT NULL DEFAULT 0,

    -- State & Memory
    context                   TEXT             NOT NULL DEFAULT '{}',
    status                    TEXT             NOT NULL DEFAULT 'active',
    user_id                   TEXT,                                                -- Stores the logged-in user ID (Nullable until login)

    -- Timestamps
    created_at                DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at                DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP, -- <--- FIXED
    expires_at                DATETIME         NOT NULL,

    FOREIGN KEY (realm_id) REFERENCES realms (id)
);

CREATE INDEX idx_auth_sessions_expires ON auth_sessions (expires_at);