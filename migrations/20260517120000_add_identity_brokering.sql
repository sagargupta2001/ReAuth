ALTER TABLE realms
    ADD COLUMN idp_broker_enabled INTEGER NOT NULL DEFAULT 0;

CREATE TABLE identity_providers (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    alias TEXT NOT NULL,
    display_name TEXT NOT NULL,
    protocol TEXT NOT NULL,
    preset_key TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    client_id TEXT NOT NULL,
    client_secret TEXT,
    issuer TEXT,
    authorization_endpoint TEXT,
    token_endpoint TEXT,
    userinfo_endpoint TEXT,
    jwks_uri TEXT,
    scopes_json TEXT NOT NULL DEFAULT '[]',
    claim_mapping_json TEXT NOT NULL DEFAULT '{}',
    pkce_required INTEGER NOT NULL DEFAULT 1,
    allow_login INTEGER NOT NULL DEFAULT 1,
    allow_link INTEGER NOT NULL DEFAULT 1,
    allow_jit_provisioning INTEGER NOT NULL DEFAULT 0,
    allow_email_auto_link INTEGER NOT NULL DEFAULT 0,
    require_verified_email INTEGER NOT NULL DEFAULT 1,
    icon_ref TEXT,
    button_color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    metadata_cached_at DATETIME,
    metadata_cache_json TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    CHECK (protocol IN ('oidc', 'oauth2')),
    UNIQUE (realm_id, alias)
);

CREATE INDEX idx_identity_providers_realm_enabled
    ON identity_providers(realm_id, enabled, sort_order);

CREATE TABLE federated_identities (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    external_username TEXT,
    external_email TEXT,
    raw_claims_json TEXT,
    linked_via TEXT NOT NULL,
    last_login_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES identity_providers(id) ON DELETE RESTRICT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (realm_id, provider_id, subject)
);

CREATE INDEX idx_federated_identities_realm_user
    ON federated_identities(realm_id, user_id);

CREATE TABLE oauth_broker_states (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    auth_session_id TEXT NOT NULL,
    pkce_verifier_hash TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    nonce TEXT,
    expires_at DATETIME NOT NULL,
    consumed_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES identity_providers(id) ON DELETE CASCADE,
    FOREIGN KEY (auth_session_id) REFERENCES auth_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_oauth_broker_states_realm_expires
    ON oauth_broker_states(realm_id, expires_at);

CREATE INDEX idx_oauth_broker_states_provider_session
    ON oauth_broker_states(provider_id, auth_session_id);
