ALTER TABLE realms ADD COLUMN pkce_required_public_clients INTEGER NOT NULL DEFAULT 1;
ALTER TABLE realms ADD COLUMN lockout_threshold INTEGER NOT NULL DEFAULT 5;
ALTER TABLE realms ADD COLUMN lockout_duration_secs INTEGER NOT NULL DEFAULT 900;

UPDATE realms
SET pkce_required_public_clients = 1
WHERE pkce_required_public_clients IS NULL;

UPDATE realms
SET lockout_threshold = 5
WHERE lockout_threshold IS NULL;

UPDATE realms
SET lockout_duration_secs = 900
WHERE lockout_duration_secs IS NULL;
