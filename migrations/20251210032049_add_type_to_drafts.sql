-- Add flow_type to drafts so we distinguish Browser vs Registration drafts
ALTER TABLE flow_drafts
    ADD COLUMN flow_type TEXT NOT NULL DEFAULT 'browser';