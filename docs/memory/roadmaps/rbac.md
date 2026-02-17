# Feature Roadmap: RBAC

## Goal (MVP)
- Provide a realm-scoped RBAC system with client roles, composite roles, hierarchical groups, and user-defined permissions.

## Current state (code-aligned)
### Core architecture
- [x] Recursive resolution engine for effective permissions (SQLx recursive CTEs).
- [x] Hybrid role scoping (realm roles + client roles in one table).
- [x] Composite roles supported in schema, queries, and management UI/API.

### Backend API
- [x] Role CRUD (create, list, get, update, delete).
- [x] Client role list endpoint (`GET /rbac/clients/{client_id}/roles`).
- [x] Context-aware filtering (realm vs client roles).
- [x] Role permission APIs (list, add, revoke, bulk).
- [x] User-role assignment endpoint (`POST /users/{id}/roles`).
- [x] Group creation endpoint.
- [x] Group delete endpoint + delete summary (cascade awareness).

### Frontend experience
- [x] Global Roles management page.
- [x] Client Roles tab in Client Details page.
- [x] Polymorphic CreateRoleForm (page + dialog).
- [x] Tab-to-route mapping for client details.
- [x] Sticky header + tabs layout.
- [x] Permission picker UI for system permissions (RolePermissionsTab).
- [x] Composite roles management tab (Role Composites).

## MVP scope (remaining, prioritized)
1. **User-defined permissions (custom permissions)**
   - Schema: add `permissions` table with `id`, `realm_id`, `client_id?`, `name`, `description`, `is_system`, `created_by`, timestamps.
   - APIs: CRUD + list/search/paginate + assign to roles (reuse existing role_permissions mapping).
   - Validation: enforce namespace rules, prevent collisions with system permissions, and scope correctness.
   - UI: extend RolePermissionsTab with a “Create custom permission” flow and list/filter of custom permissions.

## Out of MVP (later)
- RBAC audit persistence and reporting.
- Large-scale tree performance (virtualized tree, lazy loading, incremental search).
- Fine-grained authorization policies (ABAC or policy engine integration).
- Service accounts (client credentials flow + role binding).
- Audit log analytics/retention policies beyond basic storage.
- Role assignment approvals / workflow.

## Risks / dependencies
- Custom permissions require schema + migration; plan for name collisions and role assignment impact.
- Audit persistence requires a stable event schema and storage strategy.
- Alphabetical ordering means manual reordering is not supported unless explicitly added.

## Open questions
- **Answered:** Permissions should be user-defined (custom) in addition to system permissions.
- **Answered:** Group ordering remains alphabetical (manual reordering intentionally removed).
- How should custom permissions be scoped: realm-only, client-only, or both?
- Should custom permissions be deletable if assigned to roles (hard block vs cascade)?
- Should custom permissions support namespaces (e.g., `app:resource:action`) with validation rules?
