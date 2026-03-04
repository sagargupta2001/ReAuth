CREATE TABLE IF NOT EXISTS themes
(
    id          TEXT PRIMARY KEY NOT NULL,
    realm_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    description TEXT,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    UNIQUE (realm_id, name)
);

CREATE INDEX IF NOT EXISTS idx_themes_realm
    ON themes (realm_id);

CREATE TABLE IF NOT EXISTS theme_tokens
(
    id          TEXT PRIMARY KEY NOT NULL,
    theme_id    TEXT             NOT NULL,
    tokens_json TEXT             NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id)
);

CREATE INDEX IF NOT EXISTS idx_theme_tokens_theme
    ON theme_tokens (theme_id);

CREATE TABLE IF NOT EXISTS theme_layouts
(
    id          TEXT PRIMARY KEY NOT NULL,
    theme_id    TEXT             NOT NULL,
    name        TEXT             NOT NULL,
    layout_json TEXT             NOT NULL,
    created_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, name)
);

CREATE INDEX IF NOT EXISTS idx_theme_layouts_theme
    ON theme_layouts (theme_id);

CREATE TABLE IF NOT EXISTS theme_nodes
(
    id             TEXT PRIMARY KEY NOT NULL,
    theme_id       TEXT             NOT NULL,
    node_key       TEXT             NOT NULL,
    blueprint_json TEXT             NOT NULL,
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, node_key)
);

CREATE INDEX IF NOT EXISTS idx_theme_nodes_theme
    ON theme_nodes (theme_id);

CREATE TABLE IF NOT EXISTS theme_assets
(
    id         TEXT PRIMARY KEY NOT NULL,
    theme_id   TEXT             NOT NULL,
    asset_type TEXT             NOT NULL,
    filename   TEXT             NOT NULL,
    mime_type  TEXT             NOT NULL,
    byte_size  INTEGER          NOT NULL,
    checksum   TEXT,
    data       BLOB             NOT NULL,
    created_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_theme_assets_theme
    ON theme_assets (theme_id);

CREATE TABLE IF NOT EXISTS theme_versions
(
    id             TEXT PRIMARY KEY NOT NULL,
    theme_id       TEXT             NOT NULL,
    version_number INTEGER          NOT NULL,
    status         TEXT             NOT NULL DEFAULT 'draft',
    snapshot_json  TEXT             NOT NULL,
    created_at     DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    UNIQUE (theme_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_theme_versions_theme
    ON theme_versions (theme_id);

CREATE TABLE IF NOT EXISTS theme_bindings
(
    id                TEXT PRIMARY KEY NOT NULL,
    realm_id          TEXT             NOT NULL,
    client_id         TEXT,
    theme_id          TEXT             NOT NULL,
    active_version_id TEXT             NOT NULL,
    created_at        DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        DATETIME         NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (theme_id) REFERENCES themes (id) ON DELETE CASCADE,
    FOREIGN KEY (active_version_id) REFERENCES theme_versions (id) ON DELETE CASCADE,
    UNIQUE (realm_id, client_id)
);

CREATE INDEX IF NOT EXISTS idx_theme_bindings_realm
    ON theme_bindings (realm_id);
