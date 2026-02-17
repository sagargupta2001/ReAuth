-- Add custom permissions for realm/client scoped apps
CREATE TABLE custom_permissions
(
    id          TEXT NOT NULL PRIMARY KEY,
    realm_id    TEXT NOT NULL,
    client_id   TEXT NULL,
    permission  TEXT NOT NULL,
    name        TEXT NOT NULL,
    description TEXT NULL,
    created_by  TEXT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oidc_clients (id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE SET NULL,
    UNIQUE (realm_id, client_id, permission)
);

CREATE INDEX idx_custom_permissions_realm ON custom_permissions (realm_id);
CREATE INDEX idx_custom_permissions_client ON custom_permissions (client_id);
CREATE INDEX idx_custom_permissions_permission ON custom_permissions (permission);
