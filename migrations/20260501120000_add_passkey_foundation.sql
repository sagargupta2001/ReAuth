CREATE TABLE realm_passkey_settings (
    realm_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    allow_password_fallback INTEGER NOT NULL DEFAULT 1,
    discoverable_preferred INTEGER NOT NULL DEFAULT 1,
    challenge_ttl_secs INTEGER NOT NULL DEFAULT 120,
    reauth_max_age_secs INTEGER NOT NULL DEFAULT 300,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX idx_realm_passkey_settings_realm_id
    ON realm_passkey_settings(realm_id);

CREATE TABLE passkey_credentials (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    credential_id_b64url TEXT NOT NULL,
    public_key_cose_b64url TEXT NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT 0,
    transports_json TEXT NOT NULL DEFAULT '[]',
    backed_up INTEGER NOT NULL DEFAULT 0,
    backup_eligible INTEGER NOT NULL DEFAULT 0,
    aaguid TEXT,
    friendly_name TEXT,
    last_used_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (realm_id, credential_id_b64url)
);

CREATE INDEX idx_passkey_credentials_realm_user
    ON passkey_credentials(realm_id, user_id);

CREATE TABLE passkey_challenges (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    auth_session_id TEXT NOT NULL,
    user_id TEXT,
    challenge_kind TEXT NOT NULL,
    challenge_hash TEXT NOT NULL,
    rp_id TEXT NOT NULL,
    allowed_origins_json TEXT NOT NULL DEFAULT '[]',
    expires_at DATETIME NOT NULL,
    consumed_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    FOREIGN KEY (auth_session_id) REFERENCES auth_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    CHECK (challenge_kind IN ('authentication', 'enrollment', 'reauthentication'))
);

CREATE INDEX idx_passkey_challenges_realm_session_kind
    ON passkey_challenges(realm_id, auth_session_id, challenge_kind);

CREATE INDEX idx_passkey_challenges_realm_expires
    ON passkey_challenges(realm_id, expires_at);

CREATE INDEX idx_passkey_challenges_realm_consumed
    ON passkey_challenges(realm_id, consumed_at);
