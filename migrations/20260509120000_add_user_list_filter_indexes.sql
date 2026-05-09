CREATE INDEX IF NOT EXISTS idx_users_realm_created_at
    ON users(realm_id, created_at);

CREATE INDEX IF NOT EXISTS idx_users_realm_last_sign_in_at
    ON users(realm_id, last_sign_in_at);
