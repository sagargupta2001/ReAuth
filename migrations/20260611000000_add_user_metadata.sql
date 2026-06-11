ALTER TABLE users ADD COLUMN public_metadata_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE users ADD COLUMN private_metadata_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE users ADD COLUMN unsafe_metadata_json TEXT NOT NULL DEFAULT '{}';
