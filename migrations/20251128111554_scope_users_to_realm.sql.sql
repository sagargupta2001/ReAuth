-- 1. Drop the old unique index on username (if it exists as a constraint)
-- SQLite specific: We might need to recreate the table to strictly enforce FKs,
-- but for adding a column, this works for now.
-- Ideally, we drop the table and recreate it because the Unique constraint is changing.

DROP TABLE users;

CREATE TABLE users
(
    id              TEXT PRIMARY KEY NOT NULL,
    realm_id        TEXT             NOT NULL,
    username        TEXT             NOT NULL,
    hashed_password TEXT             NOT NULL,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, username) -- Scoped uniqueness
);