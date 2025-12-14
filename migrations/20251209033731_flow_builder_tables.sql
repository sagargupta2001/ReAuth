-- 1. Drafts: The editable "Save File" for the Flow Builder UI
CREATE TABLE IF NOT EXISTS flow_drafts
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    graph_json  TEXT             NOT NULL, -- The raw React Flow JSON (nodes + edges)
    created_at  TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);

-- 2. Versions: The immutable, compiled snapshots used at runtime
CREATE TABLE IF NOT EXISTS flow_versions
(
    id                 TEXT PRIMARY KEY NOT NULL, -- UUID
    draft_id           TEXT             NOT NULL,
    version_number     INTEGER          NOT NULL,
    execution_artifact TEXT             NOT NULL, -- The compiled linear execution plan (JSON)
    checksum           TEXT             NOT NULL, -- To detect drift
    created_at         TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (draft_id) REFERENCES flow_drafts (id) ON DELETE CASCADE,
    UNIQUE (draft_id, version_number)
);

-- 3. Deployments: Which version is currently active for a specific purpose?
-- e.g. Realm A's "browser_login" uses Version 5 of Draft X.
CREATE TABLE IF NOT EXISTS flow_deployments
(
    id                TEXT PRIMARY KEY NOT NULL, -- UUID
    realm_id          TEXT             NOT NULL,
    flow_type         TEXT             NOT NULL, -- e.g., 'browser', 'registration'
    active_version_id TEXT             NOT NULL,
    updated_at        TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (active_version_id) REFERENCES flow_versions (id),
    UNIQUE (realm_id, flow_type)
);