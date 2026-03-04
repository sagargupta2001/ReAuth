ALTER TABLE themes
    ADD COLUMN is_system INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_themes_system
    ON themes (is_system);
