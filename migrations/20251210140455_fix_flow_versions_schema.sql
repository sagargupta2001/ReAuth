-- 1. Disable Foreign Key checks temporarily
PRAGMA foreign_keys = OFF;

-- 2. Drop the DEPENDENT table first (Deployments point to Versions)
DROP TABLE IF EXISTS flow_deployments;

-- 3. Now it is safe to drop the Versions table
DROP TABLE IF EXISTS flow_versions;

-- 4. Re-enable Foreign Key checks
PRAGMA foreign_keys = ON;

-- 5. Recreate flow_versions (Corrected Schema)
CREATE TABLE flow_versions
(
    id                 TEXT PRIMARY KEY NOT NULL,
    flow_id            TEXT             NOT NULL,
    version_number     INTEGER          NOT NULL,
    execution_artifact TEXT             NOT NULL,
    checksum           TEXT             NOT NULL,
    created_at         DATETIME         NOT NULL,

    -- Ensure 'auth_flows' is the correct table name.
    -- If your main table is named 'flows', change this reference!
    FOREIGN KEY (flow_id) REFERENCES auth_flows (id) ON DELETE CASCADE,

    UNIQUE (flow_id, version_number)
);

-- 6. Recreate flow_deployments (Since we dropped it)
CREATE TABLE flow_deployments
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    flow_type         TEXT             NOT NULL,
    active_version_id TEXT             NOT NULL,
    updated_at        DATETIME         NOT NULL,

    FOREIGN KEY (active_version_id) REFERENCES flow_versions (id) ON DELETE CASCADE,
    UNIQUE (realm_id, flow_type)
);