-- Stores registered client applications
CREATE TABLE IF NOT EXISTS oidc_clients
(
    id            TEXT PRIMARY KEY NOT NULL,
    realm_id      TEXT             NOT NULL,
    client_id     TEXT             NOT NULL UNIQUE,
    client_secret TEXT,
    redirect_uris TEXT             NOT NULL, -- JSON array of allowed URLs
    scopes        TEXT             NOT NULL,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);

-- Stores Authorization Codes (The temporary code sent from /authorize to /token)
CREATE TABLE IF NOT EXISTS authorization_codes
(
    code                  TEXT PRIMARY KEY NOT NULL,
    user_id               TEXT             NOT NULL,
    client_id             TEXT             NOT NULL,
    redirect_uri          TEXT             NOT NULL,
    nonce                 TEXT,
    code_challenge        TEXT,
    code_challenge_method TEXT             NOT NULL, -- e.g., 'S256'
    expires_at            TIMESTAMP        NOT NULL
);