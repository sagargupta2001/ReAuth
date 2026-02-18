use serde::Serialize;

// The Raw Strings (For use in backend guards: require_permission(REALM_READ))
pub const REALM_READ: &str = "realm:read";
pub const REALM_WRITE: &str = "realm:write";
pub const REALM_DELETE: &str = "realm:delete";

pub const CLIENT_READ: &str = "client:read";
pub const CLIENT_CREATE: &str = "client:create";
pub const CLIENT_UPDATE: &str = "client:update";
pub const CLIENT_DELETE: &str = "client:delete";

pub const USER_READ: &str = "user:read";
pub const USER_WRITE: &str = "user:write";
pub const USER_DELETE: &str = "user:delete";
pub const USER_IMPERSONATE: &str = "user:impersonate"; // High privilege

pub const RBAC_READ: &str = "rbac:read";
pub const RBAC_WRITE: &str = "rbac:write"; // Manage roles/permissions

pub const SESSION_READ: &str = "session:read";
pub const SESSION_REVOKE: &str = "session:revoke";

pub const EVENT_READ: &str = "event:read"; // View Audit Logs

// Super Wildcard
pub const ALL: &str = "*";

// Metadata Structures (For the UI)
#[derive(Debug, Serialize, Clone)]
pub struct PermissionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResourceGroup {
    pub id: String,
    pub label: String,
    pub description: String,
    pub permissions: Vec<PermissionDef>,
}

// The Registry (Returns the rich UI data)
pub fn get_system_permissions() -> Vec<ResourceGroup> {
    vec![
        ResourceGroup {
            id: "realm".to_string(),
            label: "Realm Management".to_string(),
            description: "Control access to high-level realm settings and keys.".to_string(),
            permissions: vec![
                p(
                    REALM_READ,
                    "View Realm",
                    "Read-only access to general settings.",
                ),
                p(
                    REALM_WRITE,
                    "Edit Realm",
                    "Update realm configuration and keys.",
                ),
                p(
                    REALM_DELETE,
                    "Delete Realm",
                    "Critical access to destroy the realm.",
                ),
            ],
        },
        ResourceGroup {
            id: "clients".to_string(),
            label: "Clients (Applications)".to_string(),
            description: "Manage OIDC clients, secrets, and redirect URIs.".to_string(),
            permissions: vec![
                p(
                    CLIENT_READ,
                    "View Clients",
                    "List and inspect registered applications.",
                ),
                p(CLIENT_CREATE, "Create Client", "Register new applications."),
                p(
                    CLIENT_UPDATE,
                    "Edit Client",
                    "Modify client settings and secrets.",
                ),
                p(
                    CLIENT_DELETE,
                    "Delete Client",
                    "Remove applications permanently.",
                ),
            ],
        },
        ResourceGroup {
            id: "users".to_string(),
            label: "User Management".to_string(),
            description: "Control user directory, profiles, and credentials.".to_string(),
            permissions: vec![
                p(USER_READ, "View Users", "Search and view user profiles."),
                p(
                    USER_WRITE,
                    "Edit Users",
                    "Update profiles, reset passwords.",
                ),
                p(USER_DELETE, "Delete Users", "Remove users permanently."),
                p(
                    USER_IMPERSONATE,
                    "Impersonate",
                    "Login as user (for debugging).",
                ),
            ],
        },
        ResourceGroup {
            id: "rbac".to_string(),
            label: "Access Control".to_string(),
            description: "Manage roles, groups, and permissions.".to_string(),
            permissions: vec![
                p(RBAC_READ, "View RBAC", "Inspect roles and permissions."),
                p(
                    RBAC_WRITE,
                    "Manage RBAC",
                    "Create roles and assign permissions.",
                ),
            ],
        },
        ResourceGroup {
            id: "logs".to_string(),
            label: "Audit & Logs".to_string(),
            description: "View system events and user sessions.".to_string(),
            permissions: vec![
                p(EVENT_READ, "View Logs", "Access the system audit trail."),
                p(
                    SESSION_READ,
                    "View Sessions",
                    "Inspect active user sessions.",
                ),
                p(SESSION_REVOKE, "Revoke Sessions", "Force logout users."),
            ],
        },
    ]
}

/// Returns true when a permission belongs to the system registry (or is the wildcard).
///
/// # Examples
/// ```
/// use reauth_core::domain::permissions::{is_system_permission, REALM_READ};
/// assert!(is_system_permission(REALM_READ));
/// ```
pub fn is_system_permission(permission: &str) -> bool {
    if permission == ALL {
        return true;
    }

    get_system_permissions()
        .iter()
        .flat_map(|g| g.permissions.iter())
        .any(|p| p.id == permission)
}

// Helper to shorten the vector construction
fn p(id: &str, name: &str, desc: &str) -> PermissionDef {
    PermissionDef {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        custom_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_permissions_include_realm_read() {
        assert!(is_system_permission(REALM_READ));
    }

    #[test]
    fn non_system_permissions_are_rejected() {
        assert!(!is_system_permission("custom:perm"));
    }
}
