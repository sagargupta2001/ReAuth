ALTER TABLE identity_providers
    ADD COLUMN jwks_cached_at DATETIME;

ALTER TABLE identity_providers
    ADD COLUMN jwks_cache_json TEXT;
