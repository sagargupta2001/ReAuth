CREATE TABLE IF NOT EXISTS webhook_endpoints
(
    id                   TEXT PRIMARY KEY NOT NULL,
    realm_id             TEXT             NOT NULL,
    name                 TEXT             NOT NULL,
    url                  TEXT             NOT NULL,
    status               TEXT             NOT NULL DEFAULT 'active',
    signing_secret       TEXT             NOT NULL,
    custom_headers       TEXT             NOT NULL DEFAULT '{}',
    description          TEXT,
    consecutive_failures INTEGER          NOT NULL DEFAULT 0,
    last_failure_at      DATETIME,
    disabled_at          DATETIME,
    disabled_reason      TEXT,
    created_at           DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_realm
    ON webhook_endpoints (realm_id);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_status
    ON webhook_endpoints (status);

CREATE TABLE IF NOT EXISTS webhook_subscriptions
(
    endpoint_id TEXT    NOT NULL,
    event_type  TEXT    NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (endpoint_id, event_type),
    FOREIGN KEY (endpoint_id) REFERENCES webhook_endpoints (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_webhook_subscriptions_event_type
    ON webhook_subscriptions (event_type);

CREATE TABLE IF NOT EXISTS event_outbox
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT,
    event_type      TEXT             NOT NULL,
    event_version   TEXT             NOT NULL DEFAULT 'v1',
    occurred_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    payload_json    TEXT             NOT NULL,
    status          TEXT             NOT NULL DEFAULT 'pending',
    attempt_count   INTEGER          NOT NULL DEFAULT 0,
    next_attempt_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    locked_at       DATETIME,
    locked_by       TEXT,
    last_error      TEXT,
    created_at      DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_event_outbox_status_next_attempt
    ON event_outbox (status, next_attempt_at);

CREATE INDEX IF NOT EXISTS idx_event_outbox_realm_occurred_at
    ON event_outbox (realm_id, occurred_at);

CREATE INDEX IF NOT EXISTS idx_event_outbox_event_type
    ON event_outbox (event_type);
