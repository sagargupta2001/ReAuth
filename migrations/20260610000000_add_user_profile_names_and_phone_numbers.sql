ALTER TABLE users ADD COLUMN first_name TEXT;
ALTER TABLE users ADD COLUMN last_name TEXT;
ALTER TABLE users ADD COLUMN updated_at DATETIME;

CREATE TABLE user_phone_numbers (
    id                      TEXT     PRIMARY KEY NOT NULL,
    user_id                 TEXT     NOT NULL,
    realm_id                TEXT     NOT NULL,
    phone_number            TEXT     NOT NULL,
    phone_number_normalized TEXT     NOT NULL,
    is_primary              INTEGER  NOT NULL DEFAULT 0,
    is_verified             INTEGER  NOT NULL DEFAULT 0,
    created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id)  REFERENCES users (id)  ON DELETE CASCADE,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,

    UNIQUE (realm_id, phone_number_normalized)
);

CREATE TRIGGER trg_user_phone_numbers_single_primary_insert
BEFORE INSERT ON user_phone_numbers
WHEN NEW.is_primary = 1
BEGIN
    UPDATE user_phone_numbers
    SET    is_primary = 0,
           updated_at = CURRENT_TIMESTAMP
    WHERE  user_id    = NEW.user_id
    AND    is_primary = 1;
END;

CREATE TRIGGER trg_user_phone_numbers_single_primary_update
BEFORE UPDATE OF is_primary ON user_phone_numbers
WHEN NEW.is_primary = 1
BEGIN
    UPDATE user_phone_numbers
    SET    is_primary = 0,
           updated_at = CURRENT_TIMESTAMP
    WHERE  user_id    = NEW.user_id
    AND    id        != NEW.id
    AND    is_primary = 1;
END;

CREATE INDEX idx_user_phone_numbers_user_id  ON user_phone_numbers (user_id);
CREATE INDEX idx_user_phone_numbers_realm_id ON user_phone_numbers (realm_id);
