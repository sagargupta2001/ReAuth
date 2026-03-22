CREATE TABLE IF NOT EXISTS realm_recovery_settings (
    realm_id TEXT PRIMARY KEY,
    token_ttl_minutes INTEGER NOT NULL DEFAULT 15,
    rate_limit_max INTEGER NOT NULL DEFAULT 5,
    rate_limit_window_minutes INTEGER NOT NULL DEFAULT 15,
    revoke_sessions_on_reset INTEGER NOT NULL DEFAULT 1,
    email_subject TEXT,
    email_body TEXT,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_realm_recovery_settings_realm_id
    ON realm_recovery_settings(realm_id);
