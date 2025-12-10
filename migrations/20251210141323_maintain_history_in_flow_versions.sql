-- Add graph_json to flow_versions so we can restore the visual state later
ALTER TABLE flow_versions
    ADD COLUMN graph_json TEXT DEFAULT '{}' NOT NULL;