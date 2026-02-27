CREATE TABLE IF NOT EXISTS login_attempts
(
    realm_id       TEXT     NOT NULL,
    username       TEXT     NOT NULL,
    failed_count   INTEGER  NOT NULL DEFAULT 0,
    locked_until   DATETIME,
    last_failed_at DATETIME,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (realm_id, username),
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_login_attempts_realm
    ON login_attempts (realm_id);

CREATE INDEX IF NOT EXISTS idx_login_attempts_locked_until
    ON login_attempts (locked_until);
