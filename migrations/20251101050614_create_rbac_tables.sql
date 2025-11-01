-- Creates the 'roles' table
CREATE TABLE IF NOT EXISTS roles
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    name        TEXT NOT NULL UNIQUE,
    description TEXT
);

-- Creates the 'groups' table
CREATE TABLE IF NOT EXISTS groups
(
    id          TEXT PRIMARY KEY NOT NULL, -- UUID
    name        TEXT NOT NULL UNIQUE,
    description TEXT
);

-- Links roles to specific, hardcoded permission strings
CREATE TABLE IF NOT EXISTS role_permissions
(
    role_id         TEXT NOT NULL,
    permission_name TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_name),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- Composite Roles (DAG: roles can contain other roles)
CREATE TABLE IF NOT EXISTS role_composite_roles
(
    parent_role_id TEXT NOT NULL,
    child_role_id  TEXT NOT NULL,
    PRIMARY KEY (parent_role_id, child_role_id),
    FOREIGN KEY (parent_role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (child_role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- User-to-Group memberships (many-to-many)
CREATE TABLE IF NOT EXISTS user_groups
(
    user_id  TEXT NOT NULL,
    group_id TEXT NOT NULL,
    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Group-to-Role memberships (many-to-many)
CREATE TABLE IF NOT EXISTS group_roles
(
    group_id TEXT NOT NULL,
    role_id  TEXT NOT NULL,
    PRIMARY KEY (group_id, role_id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);