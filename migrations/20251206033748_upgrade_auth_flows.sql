-- Add metadata fields to auth_flows
ALTER TABLE auth_flows
    ADD COLUMN alias TEXT;
ALTER TABLE auth_flows
    ADD COLUMN type TEXT NOT NULL DEFAULT 'browser'; -- 'browser', 'registration', 'direct'
ALTER TABLE auth_flows
    ADD COLUMN built_in BOOLEAN NOT NULL DEFAULT FALSE;

-- We don't store "is_default" on the flow itself because multiple flows can be default for *different* purposes.
-- Instead, we usually store the "default_flow_id" on the Realm table, OR we add a boolean flag here if we enforce one-per-type.
-- Keycloak stores it on the Realm (e.g., realm.browser_flow_id). This is cleaner.
-- Let's add the columns to the REALMS table to track which flow is active for which purpose.

ALTER TABLE realms
    ADD COLUMN browser_flow_id TEXT;
ALTER TABLE realms
    ADD COLUMN registration_flow_id TEXT;
ALTER TABLE realms
    ADD COLUMN direct_grant_flow_id TEXT;
ALTER TABLE realms
    ADD COLUMN reset_credentials_flow_id TEXT;

-- Add foreign keys for integrity (SQLite doesn't support adding FKs in ALTER easily, but this is documentation)
-- Ideally, you'd enforce this in app logic or recreate the table if strict FKs are needed.

-- Add "requirement" and "config" to steps (as discussed previously for the visual editor)
ALTER TABLE auth_flow_steps
    ADD COLUMN requirement TEXT NOT NULL DEFAULT 'REQUIRED';
ALTER TABLE auth_flow_steps
    ADD COLUMN config TEXT;
ALTER TABLE auth_flow_steps
    ADD COLUMN parent_step_id TEXT; -- For nested flows/sub-flows