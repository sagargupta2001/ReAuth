-- Add parent/ordering support for hierarchical groups

ALTER TABLE groups
    ADD COLUMN parent_id TEXT REFERENCES groups (id) ON DELETE SET NULL;

ALTER TABLE groups
    ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups (parent_id);
CREATE INDEX IF NOT EXISTS idx_groups_parent_sort ON groups (parent_id, sort_order);
