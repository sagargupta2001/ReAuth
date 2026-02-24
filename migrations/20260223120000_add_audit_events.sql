CREATE TABLE IF NOT EXISTS audit_events
(
    id             TEXT PRIMARY KEY NOT NULL,
    realm_id       TEXT             NOT NULL,
    actor_user_id  TEXT,
    action         TEXT             NOT NULL,
    target_type    TEXT             NOT NULL,
    target_id      TEXT,
    metadata       TEXT             NOT NULL DEFAULT '{}',
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (actor_user_id) REFERENCES users (id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_events_realm_created_at
    ON audit_events (realm_id, created_at);

CREATE INDEX IF NOT EXISTS idx_audit_events_actor
    ON audit_events (actor_user_id);

CREATE INDEX IF NOT EXISTS idx_audit_events_action
    ON audit_events (action);
