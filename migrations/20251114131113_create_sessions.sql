-- Add migration script here
CREATE TABLE IF NOT EXISTS refresh_tokens
(
    id         TEXT PRIMARY KEY NOT NULL,
    user_id    TEXT             NOT NULL,
    realm_id   TEXT             NOT NULL,
    expires_at TIMESTAMP        NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);