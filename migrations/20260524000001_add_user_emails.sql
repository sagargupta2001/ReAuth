-- Drop the unique index on (realm_id, email) before dropping the column.
DROP INDEX IF EXISTS users_realm_email_unique;

-- Remove the flat email column from users (DB can be wiped so no backfill needed).
-- SQLite does not support DROP COLUMN before 3.35, but sqlx ships with a modern SQLite.
ALTER TABLE users DROP COLUMN email;

-- Dedicated user_emails table supporting multiple emails per user.
CREATE TABLE user_emails (
    id               TEXT     PRIMARY KEY NOT NULL,
    user_id          TEXT     NOT NULL,
    realm_id         TEXT     NOT NULL,
    email            TEXT     NOT NULL,
    email_normalized TEXT     NOT NULL,
    is_primary       INTEGER  NOT NULL DEFAULT 0,
    is_verified      INTEGER  NOT NULL DEFAULT 0,
    created_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id)  REFERENCES users (id)  ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,

    -- Global uniqueness within a realm: one row per normalised address per realm
    UNIQUE (realm_id, email_normalized)
);

-- Trigger: when a new primary is inserted, demote any existing primary for that user
CREATE TRIGGER trg_user_emails_single_primary_insert
BEFORE INSERT ON user_emails
WHEN NEW.is_primary = 1
BEGIN
    UPDATE user_emails
    SET    is_primary = 0,
           updated_at = CURRENT_TIMESTAMP
    WHERE  user_id    = NEW.user_id
    AND    is_primary = 1;
END;

-- Trigger: same but for UPDATE ... SET is_primary = 1
CREATE TRIGGER trg_user_emails_single_primary_update
BEFORE UPDATE OF is_primary ON user_emails
WHEN NEW.is_primary = 1
BEGIN
    UPDATE user_emails
    SET    is_primary = 0,
           updated_at = CURRENT_TIMESTAMP
    WHERE  user_id    = NEW.user_id
    AND    id        != NEW.id
    AND    is_primary = 1;
END;

CREATE INDEX idx_user_emails_user_id  ON user_emails (user_id);
CREATE INDEX idx_user_emails_realm_id ON user_emails (realm_id);
