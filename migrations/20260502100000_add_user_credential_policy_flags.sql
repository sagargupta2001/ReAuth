ALTER TABLE users
    ADD COLUMN force_password_reset INTEGER NOT NULL DEFAULT 0;

ALTER TABLE users
    ADD COLUMN password_login_disabled INTEGER NOT NULL DEFAULT 0;
