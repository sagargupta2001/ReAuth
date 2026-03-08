CREATE TABLE oidc_clients_new
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    client_id         TEXT             NOT NULL,
    client_secret     TEXT,
    redirect_uris     TEXT             NOT NULL,
    scopes            TEXT             NOT NULL,
    web_origins       TEXT             NOT NULL DEFAULT '[]',
    managed_by_config BOOLEAN          NOT NULL DEFAULT FALSE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, client_id)
);

INSERT INTO oidc_clients_new (
    id,
    realm_id,
    client_id,
    client_secret,
    redirect_uris,
    scopes,
    web_origins,
    managed_by_config
)
SELECT
    id,
    realm_id,
    client_id,
    client_secret,
    redirect_uris,
    scopes,
    web_origins,
    COALESCE(managed_by_config, FALSE)
FROM oidc_clients;

DROP TABLE oidc_clients;

ALTER TABLE oidc_clients_new RENAME TO oidc_clients;
