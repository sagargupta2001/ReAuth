# RBAC Flow (ReAuth)

## Purpose
Provide a single, code-aligned reference for how RBAC entities relate, how permissions are resolved, and where performance/caching behavior lives.

## Core Entities
| Entity | Description |
| --- | --- |
| `roles` | Realm or client scoped role. `client_id = NULL` means realm role; non-NULL means client role. |
| `groups` | Hierarchical groups with `parent_id` and `sort_order`. |
| `users` | Realm-scoped user accounts. |
| `user_roles` | Direct role assignments to users. |
| `group_roles` | Direct role assignments to groups. |
| `user_groups` | Direct group memberships for users. |
| `role_composite_roles` | Role inheritance (parent -> child). |
| `role_permissions` | Mapping between roles and permission IDs. |
| `custom_permissions` | User-defined permissions scoped to realm or client. |

## Relationship Summary
| Relationship | Meaning |
| --- | --- |
| User -> Role | Direct role assignment (`user_roles`). |
| User -> Group | Direct group membership (`user_groups`). |
| Group -> Role | Direct group role assignment (`group_roles`). |
| Role -> Role | Composite inheritance (`role_composite_roles`). |
| Role -> Permission | Permission assignment (`role_permissions`). |
| Custom Permission -> Realm/Client | Scope via `realm_id` + optional `client_id`. |

## Permission Resolution Semantics
| Concept | Rule |
| --- | --- |
| Direct role | Assigned via `user_roles` or `group_roles`. |
| Effective role | Direct roles + composite expansion (recursive CTE). |
| Effective permission | Union of permissions for all effective roles. |
| Client role | Only accepts client-scoped custom permissions. |
| Realm role | Can accept system permissions and realm-scoped custom permissions. |

## Custom Permissions
- Stored in `custom_permissions` with unique `(realm_id, client_id, permission)`.
- Permission keys must be namespaced (e.g., `billing:invoices:read`).
- Wildcard `*` is reserved for system usage.
- System permissions are defined in `domain/permissions.rs` and are not stored in DB.
- Custom permissions are exposed in the UI under a “Custom Permissions” group.

## Caching
| Cache | What is cached | Invalidation triggers |
| --- | --- | --- |
| User permissions | Effective permission set per user | User role changes, group membership changes, group role changes, role permission changes, composite role changes, role deletion, group deletion |

Cache invalidation is handled by `CacheInvalidator` on domain events. The cache is authoritative for permission checks but not for role/group lists.

## Recursive CTEs (Key Flows)
| Use Case | Description |
| --- | --- |
| Effective permissions | Expand direct user roles + group roles via composite recursion, then collect role permissions. |
| Role members | Resolve users who are direct or effective members of a role via composite expansion. |
| Group subtree | Recursive traversal for delete summary and cascade operations. |

## Indexes & Optimization Notes
| Index | Purpose |
| --- | --- |
| `idx_group_roles_group` | Fast lookup of roles by group. |
| `idx_user_roles_user` | Fast lookup of roles by user. |
| `idx_role_composite_parent` | Recursive CTE parent traversal. |
| `idx_role_composite_child` | Reverse traversal / faster joins. |
| `idx_groups_parent` | Tree traversal. |
| `idx_groups_parent_sort` | Ordered children queries. |
| `idx_custom_permissions_realm` | Realm-scoped lookups. |
| `idx_custom_permissions_client` | Client-scoped lookups. |
| `idx_custom_permissions_permission` | Uniqueness + direct key lookup. |

## API Surface (RBAC)
| Endpoint | Purpose |
| --- | --- |
| `GET /rbac/permissions` | List system + custom permissions (client-scoped if `client_id` query provided). |
| `POST /rbac/permissions/custom` | Create a custom permission. |
| `PUT /rbac/permissions/custom/:id` | Update custom permission name/description. |
| `DELETE /rbac/permissions/custom/:id` | Delete custom permission (removes role mappings). |
| `GET /rbac/roles/:id/permissions` | List role permissions. |
| `POST /rbac/roles/:id/permissions` | Assign permission to role. |
| `DELETE /rbac/roles/:id/permissions` | Revoke permission from role. |
| `POST /rbac/roles/:id/permissions/bulk` | Bulk assign/remove permissions. |
| `GET /rbac/roles/:id/composites` | List composite role IDs. |
| `GET /rbac/roles/:id/composites/list` | List composites with direct/effective flags. |
| `POST /rbac/roles/:id/composites` | Assign child role. |
| `DELETE /rbac/roles/:id/composites/:child_id` | Remove child role. |
| `GET /rbac/groups/:id/roles` | List group role IDs (direct/effective via `scope`). |
| `GET /rbac/groups/:id/roles/list` | Paginated list with direct/effective flags. |
| `GET /users/:id/roles` | List user role IDs (direct/effective via `scope`). |
| `GET /users/:id/roles/list` | Paginated list with direct/effective flags. |

## Known Constraints
- Group ordering is alphabetical; manual ordering is intentionally disabled.
- System permissions cannot be assigned to client roles.
- Custom permissions must exist in scope before assignment.
