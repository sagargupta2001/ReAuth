-- Add client_id to refresh_tokens.
-- It is NULLable because internal admin logins might not have an OIDC client.
ALTER TABLE refresh_tokens
    ADD COLUMN client_id TEXT;