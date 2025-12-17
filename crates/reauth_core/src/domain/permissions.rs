pub mod permissions {
    // Client Management
    pub const CLIENT_READ: &str = "client:read";
    pub const CLIENT_CREATE: &str = "client:create";
    pub const CLIENT_UPDATE: &str = "client:update";

    // Realm Management
    pub const REALM_READ: &str = "realm:read";
    pub const REALM_WRITE: &str = "realm:write"; // covers create/update

    // RBAC Management (The "Meta" permissions)
    pub const RBAC_READ: &str = "rbac:read";
    pub const RBAC_WRITE: &str = "rbac:write";

    // User Management
    pub const USER_READ: &str = "user:read";
    pub const USER_WRITE: &str = "user:write";
}
