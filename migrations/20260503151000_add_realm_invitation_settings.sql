ALTER TABLE realms
    ADD COLUMN invitation_flow_id TEXT;

ALTER TABLE realms
    ADD COLUMN invitation_resend_limit INTEGER NOT NULL DEFAULT 3;
