# Spec: Roles Table Create Dialog

> Distilled from: Roles listing UX request / 2026-06-12
> Status: Implemented

---

## User Story

As a realm admin, I want role creation to happen from the Roles table toolbar so that searching and creating roles use the same listing workflow as Users.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Creates roles and scans role inventory from the Roles page. |

---

## Business Rules

1. The Roles page create action appears in the same toolbar row as table search.
2. Creating a realm role opens a dialog from the Roles page instead of navigating to a standalone create page.
3. Successful dialog submission closes the dialog and refreshes the roles list without redirecting away from the listing.
4. The standalone `/:realm/roles/new` page is removed from app routing.
5. The Roles table shows role scope, direct user count, permission count, and created date in addition to role name and description.
6. Role permission add, remove, bulk select-all, and custom permission deletion refresh role list data so `permission_count` stays current without a hard reload.
7. Permission resource select-all uses a radio-style toggle button instead of a checkbox.

**Edge cases:**
- The create form still supports client role creation when embedded in client-specific role surfaces.
- Rows without created timestamps render a neutral placeholder.

---

## Domain Changes

### Modified Entities

```text
Role
  + created_at: Option<String> - existing database timestamp exposed in API responses.
  + user_count: Option<i64> - direct user assignment count on list responses.
  + permission_count: Option<i64> - assigned permission count on list responses.
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/role.rs` | Expose optional list metadata fields on Role serialization. |
| `adapters/persistence/sqlite_rbac_repository.rs` | Return direct user counts with role list rows. |
| `ui/src/features/roles` | Add dialog create flow and richer table columns. |
| `ui/src/app/routerConfig.tsx` | Remove the standalone create-role route. |

---

## Persistence Changes

### New Migration(s)

```text
None.
```

### Data Notes

- `roles.created_at` already exists in the initial schema.
- `user_count` is computed from direct `user_roles` assignments at list time.
- `permission_count` is computed from `role_permissions` assignments at list time.

---

## API Changes

### Modified Endpoints

```text
GET /api/realms/{realm}/rbac/roles?page=1&per_page=10&q=admin
  Changed response rows: { created_at?: string, user_count?: number, permission_count?: number }

GET /api/realms/{realm}/rbac/clients/{client_id}/roles?page=1&per_page=10
  Changed response rows: { created_at?: string, user_count?: number, permission_count?: number }
```

---

## Flow / Auth Impact

- Flow types affected: none.

---

## Availability / Admin UX

- System/operator prerequisites: none.
- Realm policy: existing RBAC permissions continue to gate role creation.
- Simple mode UX: create role from the listing toolbar.
- Advanced mode UX: existing role detail tabs remain unchanged.

---

## Test Scenarios

1. **Dialog create**
   - Given: a realm admin is on the Roles page
   - When: they click Create Role
   - Then: a dialog opens and successful submit closes it while refreshing the table.

2. **Validation failure**
   - Given: the create dialog is open
   - When: the role name violates the existing schema
   - Then: validation appears inline and no request is sent.

3. **Richer table rows**
   - Given: roles with created timestamps, direct user assignments, and assigned permissions
   - When: the Roles table loads
   - Then: scope, direct user count, permission count, and created date are visible.

4. **Permission count refresh**
   - Given: a role is visible in the Roles table
   - When: permissions are added or removed inside the role permissions tab
   - Then: returning to the Roles table shows the updated permission count without a hard reload.

5. **Select all control**
   - Given: a permission resource section is visible
   - When: the admin uses Select All
   - Then: the control is a radio-style toggle and bulk assigns or clears the resource permissions.

6. **Removed page**
   - Given: app routing is configured
   - When: static routes are evaluated
   - Then: `/:realm/roles/new` is not registered.

---

## Out of Scope

- Group assignment counts.
- Opening the create-role dialog from global command palette actions.
