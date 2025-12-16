-- Add web_origins column, default to empty JSON array
ALTER TABLE oidc_clients
    ADD COLUMN web_origins TEXT NOT NULL DEFAULT '[]';