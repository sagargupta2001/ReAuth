CREATE TABLE IF NOT EXISTS realm_security_headers (
    realm_id TEXT PRIMARY KEY,
    x_frame_options TEXT,
    content_security_policy TEXT,
    x_content_type_options TEXT,
    referrer_policy TEXT,
    strict_transport_security TEXT,
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_realm_security_headers_realm_id
    ON realm_security_headers(realm_id);
