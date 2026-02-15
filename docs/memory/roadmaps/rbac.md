# Feature Roadmap: RBAC

## Goal
- Provide a realm-scoped RBAC system with client roles, composite roles, and auditable permission changes.

## Current state (code-aligned)
### Core architecture
- [x] Recursive resolution engine for effective permissions (SQLx recursive CTEs).
- [x] Hybrid role scoping (realm roles + client roles in one table).
- [x] Composite roles supported in schema and recursive queries (no management UI/API yet).

### Backend API
- [x] Role CRUD (create, list, get, update, delete).
- [x] Client role list endpoint (`GET /rbac/clients/{client_id}/roles`).
- [x] Context-aware filtering (realm vs client roles).
- [x] Role permission APIs (list, add, revoke, bulk).
- [x] User-role assignment endpoint (`POST /users/{id}/roles`).
- [x] Group creation endpoint.

### Frontend experience
- [x] Global Roles management page.
- [x] Client Roles tab in Client Details page.
- [x] Polymorphic CreateRoleForm (page + dialog).
- [x] Tab-to-route mapping for client details.
- [x] Sticky header + tabs layout.
- [x] Permission picker UI for system permissions (RolePermissionsTab).

## Now
- [ ] Add UI for user-role mapping (role members tab is currently placeholder).
- [ ] Add UI for group management (create, list, and membership assignment).
- [ ] Add API endpoints for group membership and role-group assignment to match UI.

## Next
- [ ] Composite roles management (API + UI): add/remove child roles.
- [ ] Distinguish direct vs effective roles in UI (user details + role members).
- [ ] Add delete group endpoint and safe delete workflows (confirmations, cascade impact).
- [ ] Add “custom permissions” model (schema + API) or clarify that permissions are system-defined only.

## Later
- [ ] Hierarchical groups (schema needs parent_id + tree UI).
- [ ] Fine-grained authorization policies (ABAC or policy engine integration).
- [ ] Service accounts (client credentials flow + role binding).
- [ ] Audit log persistence for RBAC changes (who/when/what) beyond in-memory events.

## Risks / dependencies
- Composite roles exist in schema and queries but lack management interfaces.
- Custom permissions are not modeled in DB; current UI uses system-defined permissions only.
- Group hierarchy requires schema changes.

## Open questions
- Should permissions be system-only or allow custom per-client definitions?
- Do we need audit log storage before exposing RBAC to enterprise users?
