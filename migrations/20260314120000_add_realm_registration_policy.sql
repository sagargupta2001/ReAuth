ALTER TABLE realms ADD COLUMN is_system INTEGER NOT NULL DEFAULT 0;
ALTER TABLE realms ADD COLUMN registration_enabled INTEGER NOT NULL DEFAULT 1;
ALTER TABLE realms ADD COLUMN default_registration_role_ids TEXT NOT NULL DEFAULT '[]';

UPDATE realms
SET is_system = 1,
    registration_enabled = 0
WHERE name = 'master';
