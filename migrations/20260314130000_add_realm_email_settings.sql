CREATE TABLE IF NOT EXISTS realm_email_settings (
    realm_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    from_address TEXT,
    from_name TEXT,
    reply_to_address TEXT,
    smtp_host TEXT,
    smtp_port INTEGER,
    smtp_username TEXT,
    smtp_password TEXT,
    smtp_security TEXT NOT NULL DEFAULT 'starttls',
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_realm_email_settings_realm_id
    ON realm_email_settings(realm_id);
