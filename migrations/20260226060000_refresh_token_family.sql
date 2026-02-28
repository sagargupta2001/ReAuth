-- Refresh token family tracking + rotation metadata
ALTER TABLE refresh_tokens ADD COLUMN family_id TEXT;
ALTER TABLE refresh_tokens ADD COLUMN revoked_at DATETIME;
ALTER TABLE refresh_tokens ADD COLUMN replaced_by TEXT;

-- Backfill existing tokens to their own families
UPDATE refresh_tokens SET family_id = id WHERE family_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family_id ON refresh_tokens (family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_revoked_at ON refresh_tokens (revoked_at);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_replaced_by ON refresh_tokens (replaced_by);
