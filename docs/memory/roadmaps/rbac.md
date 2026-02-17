# Feature Roadmap: RBAC

## Goal
- Provide a realm-scoped RBAC system with client roles, composite roles, and auditable permission changes.

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

### Frontend experience
- [x] Global Roles management page.
- [x] Client Roles tab in Client Details page.
- [x] Polymorphic CreateRoleForm (page + dialog).
- [x] Tab-to-route mapping for client details.
- [x] Sticky header + tabs layout.
- [x] Permission picker UI for system permissions (RolePermissionsTab).

## Now
- [x] Add UI for user-role mapping (role members tab).
- [x] Add UI for group management (create, list, edit, assign members/roles).
- [x] Add API endpoints for group membership and role-group assignment to match UI.
- [x] Add server-side filtering/sorting/pagination for group and members lists.
- [x] Add hierarchical groups schema (parent_id, sort_order, indices).
- [x] Split-pane Group Explorer UX for hierarchical groups (tree on left, tabs on right).
- [x] Tree drag-and-drop for reparenting (drop-on-node) with alphabetical ordering.
- [x] Tree state persistence (expanded nodes) and auto-expand ancestor chain on load.
- [x] Group creation updates tree immediately without full refresh.

## Next
- [x] Group tree APIs (tree/list roots + children, move/reorder).
- [x] Split-pane Group Explorer UX with drag-and-drop and modular FSD components.
- [x] Delete group endpoint with subtree safeguards (confirmations, cascade impact).
- [x] Composite roles management (API + UI): add/remove child roles.
- [x] Distinguish direct vs effective roles in UI (group + user roles).
- [ ] Add “custom permissions” model (schema + API) or clarify that permissions are system-defined only.

## Later
- [ ] Large-scale tree performance (virtualized tree, lazy loading, incremental search).
- [ ] Fine-grained authorization policies (ABAC or policy engine integration).
- [ ] Service accounts (client credentials flow + role binding).
- [ ] Audit log persistence for RBAC changes (who/when/what) beyond in-memory events.

## Risks / dependencies
- Composite roles exist in schema and queries but lack management interfaces.
- Custom permissions are not modeled in DB; current UI uses system-defined permissions only.
- Group hierarchy requires schema changes and cycle-safe move semantics.
- Drag-and-drop tree requires careful bundle sizing and performance testing.
- Alphabetical ordering means manual reordering is not supported unless explicitly added.

## Open questions
- Should permissions be system-only or allow custom per-client definitions?
- Do we need audit log storage before exposing RBAC to enterprise users?
- Should group moves be soft-validated (warn on large subtree moves) or hard-blocked?
- Should group ordering remain alphabetical or allow manual ordering within a parent?
