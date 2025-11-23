-- Add migration script here
CREATE TABLE IF NOT EXISTS realms
(
    id                     TEXT PRIMARY KEY NOT NULL,               -- UUID
    name                   TEXT             NOT NULL UNIQUE,
    access_token_ttl_secs  INTEGER          NOT NULL DEFAULT 900,   -- 15 minutes
    refresh_token_ttl_secs INTEGER          NOT NULL DEFAULT 604800 -- 7 days
);