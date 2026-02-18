ALTER TABLE oidc_clients
ADD COLUMN managed_by_config INTEGER NOT NULL DEFAULT 0;
