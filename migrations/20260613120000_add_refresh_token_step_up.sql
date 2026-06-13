-- Forced re-authentication ("step-up") for sessions.
-- When set, the live token of a refresh-token family must re-authenticate:
-- silent refresh is rejected until a fresh interactive auth mints a new family.
ALTER TABLE refresh_tokens ADD COLUMN step_up_at DATETIME NULL;
