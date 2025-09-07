-- Your SQL goes here
CREATE TABLE users
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    role     TEXT NOT NULL
);
