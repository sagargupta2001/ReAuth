ALTER TABLE realms
    ADD COLUMN idp_default_jit_policy TEXT NOT NULL DEFAULT 'per_provider';

ALTER TABLE realms
    ADD COLUMN idp_default_email_link_policy TEXT NOT NULL DEFAULT 'manual_only';

ALTER TABLE realms
    ADD COLUMN idp_minimum_remaining_factor INTEGER NOT NULL DEFAULT 1;

UPDATE realms
SET idp_default_jit_policy = COALESCE(idp_default_jit_policy, 'per_provider'),
    idp_default_email_link_policy = COALESCE(idp_default_email_link_policy, 'manual_only'),
    idp_minimum_remaining_factor = COALESCE(idp_minimum_remaining_factor, 1);
