-- 1. Drop the old table (It only holds temporary sessions, so wiping it is safe)
DROP TABLE IF EXISTS refresh_tokens;

-- 2. Recreate it with the new columns and correct defaults
CREATE TABLE refresh_tokens
(
    id           TEXT PRIMARY KEY NOT NULL,
    user_id      TEXT             NOT NULL,
    realm_id     TEXT             NOT NULL,
    client_id    TEXT,
    expires_at   TIMESTAMP        NOT NULL,

    -- New Columns
    ip_address   TEXT,
    user_agent   TEXT,
    created_at   TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);