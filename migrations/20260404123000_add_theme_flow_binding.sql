ALTER TABLE themes ADD COLUMN flow_binding_id TEXT;

CREATE INDEX IF NOT EXISTS idx_themes_flow_binding
    ON themes (flow_binding_id);
