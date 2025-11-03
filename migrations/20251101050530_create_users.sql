CREATE TABLE IF NOT EXISTS users
(
    id       TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
);