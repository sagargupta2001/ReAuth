DROP INDEX IF EXISTS idx_themes_flow_binding;

ALTER TABLE themes DROP COLUMN flow_binding_id;
