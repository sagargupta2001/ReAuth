CREATE TABLE IF NOT EXISTS recovery_attempts (
    realm_id TEXT NOT NULL,
    identifier TEXT NOT NULL,
    window_started_at DATETIME NOT NULL,
    attempt_count INTEGER NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (realm_id, identifier),
    FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recovery_attempts_realm
    ON recovery_attempts(realm_id);
