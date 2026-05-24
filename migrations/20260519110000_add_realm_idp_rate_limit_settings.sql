CREATE TABLE realm_idp_settings (
    realm_id TEXT PRIMARY KEY,
    oauth_start_rate_limit_max INTEGER NOT NULL DEFAULT 30,
    oauth_start_rate_limit_window_minutes INTEGER NOT NULL DEFAULT 10,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX idx_realm_idp_settings_realm_id
    ON realm_idp_settings(realm_id);

CREATE TABLE oauth_start_attempts (
    realm_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    window_started_at DATETIME NOT NULL,
    attempt_count INTEGER NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (realm_id, provider_id, ip_address),
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES identity_providers(id) ON DELETE CASCADE
);

CREATE INDEX idx_oauth_start_attempts_provider
    ON oauth_start_attempts(realm_id, provider_id);
