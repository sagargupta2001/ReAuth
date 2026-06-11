# Spec: User Profile Metadata

> Distilled from: Clerk-style user metadata request and supplied UI snippet on 2026-06-11
> Status: Implemented

---

## User Story

As a realm admin, I want each user to have structured metadata split by visibility so that product teams can attach application-specific JSON to users without mixing public, private, and end-user-editable data.

---

## Actors

| Actor | Role in this feature |
|-------|----------------------|
| Realm Admin | Reads and edits all user metadata types from the user profile tab |
| End User | Can read public and unsafe metadata about their own account through frontend-safe APIs, and can update unsafe metadata when exposed through self-service APIs |
| Backend API Consumer | Can read and modify all metadata types through privileged/admin APIs |
| Frontend API Consumer | Can read public metadata and read/write unsafe metadata only; cannot read private metadata |
| OIDC Client | No direct impact in this slice |
| Operator | No direct impact beyond normal database backup/retention |

---

## Business Rules

1. Each user has three metadata JSON objects:
   - `public_metadata`: readable by backend/admin APIs and frontend-safe APIs.
   - `private_metadata`: readable only by backend/admin APIs.
   - `unsafe_metadata`: readable and writable by backend/admin APIs and frontend-safe APIs.
2. Metadata values must be JSON objects. Arrays, strings, numbers, booleans, and null are rejected at the top level.
3. Empty metadata is represented as `{}`.
4. Admins with user write permission may edit all three metadata objects for any user in the realm.
5. Frontend-safe/self-service APIs must never return `private_metadata`.
6. Frontend-safe/self-service APIs may update only `unsafe_metadata`.
7. Updating any metadata object refreshes the user's `updated_at` timestamp.
8. Metadata updates replace the whole selected metadata object in v1. Deep merge/patch semantics are out of scope.
9. Metadata is not trusted input. Server-side authorization and business logic must not rely on `unsafe_metadata` for security decisions.
10. Metadata payloads are size-bounded to avoid unbounded user row growth.
11. The admin profile UI shows a `Metadata` section below phone numbers with three rows: Public, Private, and Unsafe.
12. Each row shows a compact, pretty-printed JSON preview and an `Edit` action.
13. Clicking `Edit` opens a small dialog with a JSON editor that supports indentation, syntax highlighting, and realtime parse/shape errors before save.
14. Invalid JSON or non-object JSON disables save and shows an inline error.

**Edge cases:**
- Missing database values are treated as `{}` for backward compatibility.
- Whitespace-only editor contents are treated as invalid; admins should use `{}` to clear metadata.
- Large JSON objects should remain scrollable in both preview and dialog.
- Private metadata must not leak through `/me`, public profile, OIDC userinfo, search, list users, logs, or frontend caches unless explicitly added in a future spec.

---

## Domain Changes

### New Entities

None.

### Modified Entities

```text
User
  + public_metadata: serde_json::Value/Object
  + private_metadata: serde_json::Value/Object
  + unsafe_metadata: serde_json::Value/Object
```

### New Value Objects

```text
UserMetadataVisibility = "public" | "private" | "unsafe"
UserMetadataUpdate = { metadata: object }
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/user.rs` | Add metadata fields or a typed metadata projection with safe defaults |
| `application/user_service.rs` | Add validation and update methods for metadata by visibility |
| `ports/user_repository.rs` | Persist metadata columns and update selected metadata object |
| `adapters/persistence/sqlite_user_repository.rs` | Read/write JSON text columns and preserve user `updated_at` |
| `adapters/web/user_handler.rs` | Add admin metadata endpoints and ensure response redaction by API surface |
| `adapters/web/auth_handler.rs` / self APIs | Expose only public and unsafe metadata where current-user frontend-safe responses are returned |
| `ui/src/entities/user/model/types.ts` | Add metadata types with public/private/unsafe shape |
| `ui/src/features/user/api/...` | Add metadata query/mutation hooks with correct invalidation |
| `ui/src/features/user/components/profile/...` | Add metadata section and JSON edit dialog below phone numbers |

---

## Persistence Changes

### New Migration(s)

```text
20260611000000_add_user_metadata.sql
```

### Data Notes

- Add nullable or defaulted JSON text columns to `users`:
  - `public_metadata_json TEXT NOT NULL DEFAULT '{}'`
  - `private_metadata_json TEXT NOT NULL DEFAULT '{}'`
  - `unsafe_metadata_json TEXT NOT NULL DEFAULT '{}'`
- Validate JSON object shape at the application layer. SQLite JSON functions may be used when available, but app-layer validation is the source of truth.
- Store canonical compact JSON or pretty JSON consistently; API responses return parsed JSON objects.
- Recommended initial max size: 16 KiB per metadata object after serialization. If another repo-wide payload limit exists at implementation time, use the established local limit instead.

---

## API Changes

### New HTTP Endpoints

```text
GET /api/realms/{realm}/users/{id}/metadata
  Response: {
    public_metadata: object,
    private_metadata: object,
    unsafe_metadata: object
  }
  Auth: user:read

PUT /api/realms/{realm}/users/{id}/metadata/public
  Request:  { metadata: object }
  Response: { public_metadata: object, updated_at: string }
  Auth: user:write

PUT /api/realms/{realm}/users/{id}/metadata/private
  Request:  { metadata: object }
  Response: { private_metadata: object, updated_at: string }
  Auth: user:write

PUT /api/realms/{realm}/users/{id}/metadata/unsafe
  Request:  { metadata: object }
  Response: { unsafe_metadata: object, updated_at: string }
  Auth: user:write

GET /api/realms/{realm}/users/me/metadata
  Response: {
    public_metadata: object,
    unsafe_metadata: object
  }
  Auth: authenticated user

PUT /api/realms/{realm}/users/me/metadata/unsafe
  Request:  { metadata: object }
  Response: { unsafe_metadata: object, updated_at: string }
  Auth: authenticated user
```

### Modified Endpoints

```text
GET /api/realms/{realm}/users/{id}
  Changed response: include all three metadata objects for admin/backend user detail responses.

PUT /api/realms/{realm}/users/{id}
  No metadata fields accepted here; metadata uses dedicated sub-resource endpoints.

GET /api/realms/{realm}/users/me
  Changed response: include public_metadata and unsafe_metadata only. Do not include private_metadata.
```

---

## Flow / Auth Impact

- Flow types affected: none
- New nodes: none
- Existing nodes modified: none
- Async pause/resume impact: none
- Theme or Fluid page impact: none

Metadata must not alter login, registration, recovery, or OIDC claim behavior in this slice.

### Answered Design Decisions

- Public metadata is authenticated-only by default. In this spec, `public` means frontend-safe for authenticated clients, not anonymously readable. If ReAuth adds anonymous public profiles later, that feature must use a separate metadata bucket or explicit per-key opt-in.
- OIDC custom claims may map selected public metadata keys in a future feature, but only through explicit admin-configured opt-in mapping. No metadata is embedded into OIDC tokens by default in this slice.
- Private metadata access should be easy to split into a future granular permission such as `user:read:private_metadata`. V1 uses the existing `user:read` behavior, but private metadata redaction is isolated at the application response-construction layer rather than baked into route or repository boundaries.

---

## Availability / Admin UX

- System/operator prerequisites: none
- Realm policy: none in this slice
- Flow composition: none
- Builder behavior: none
- Simple mode UX: user profile tab shows a Metadata section under phone numbers.
- Advanced mode UX: none

### Admin UI Details

The `Metadata` section should visually follow the supplied reference:

- A single card titled `Metadata`.
- Three stacked rows: `Public`, `Private`, `Unsafe`.
- Public and Unsafe use an eye-style icon; Private uses a lock-style icon.
- Each row has an `Edit` button aligned right.
- Each row previews the JSON as pretty-printed code. Empty objects render as `{}`.
- The edit dialog contains:
  - title matching the metadata type
  - short visibility explanation
  - JSON editor with syntax highlighting and indentation
  - realtime parse errors
  - realtime top-level object validation
  - Cancel and Save actions
- Use existing UI primitives and existing editor dependencies where possible. The repo already includes CodeMirror JSON packages, so prefer that over a new editor dependency unless implementation discovers a stronger local pattern.

---

## Test Scenarios

1. **Admin reads all metadata**
   - Given: a user has public, private, and unsafe metadata
   - When: an admin with `user:read` requests user metadata
   - Then: all three metadata objects are returned.

2. **Frontend-safe read redacts private metadata**
   - Given: a user has all three metadata objects
   - When: the current-user/frontend-safe metadata endpoint is requested
   - Then: public and unsafe metadata are returned and private metadata is absent.

3. **Admin update**
   - Given: an admin with `user:write`
   - When: the admin updates private metadata with a valid JSON object
   - Then: the private metadata is persisted and `updated_at` changes.

4. **Unsafe self update**
   - Given: an authenticated user
   - When: they update their unsafe metadata with a valid JSON object
   - Then: only `unsafe_metadata` changes; public and private metadata are unchanged.

5. **Validation failure**
   - Given: a metadata update request
   - When: the payload is invalid JSON, a top-level array, or exceeds the size limit
   - Then: the API returns a field validation error and no metadata is changed.

6. **UI editor validation**
   - Given: the admin opens the metadata edit dialog
   - When: they type malformed JSON or a non-object JSON value
   - Then: the dialog shows realtime errors and Save is disabled.

7. **UI preview**
   - Given: metadata exists
   - When: the profile tab loads
   - Then: each metadata type renders as pretty-printed JSON in the Metadata card below phone numbers.

---

## Out of Scope

- OIDC/userinfo claims for metadata.
- JWT/session token embedding of metadata.
- Metadata schema definitions or typed validation per realm.
- Deep merge/JSON Patch updates.
- Metadata audit diff viewer.
- Searching/filtering users by metadata.
- Admin permissions split by metadata type.

---

## Open Questions

- [x] Should public metadata be exposed to unauthenticated public profile endpoints if ReAuth adds such endpoints later? No. Keep it authenticated-only unless a future feature adds a separate anonymous bucket or explicit per-key opt-in.
- [x] Should future OIDC custom claims be able to map selected public metadata keys? Yes, but only by explicit admin-configured opt-in mapping and not in v1.
- [x] Should there be separate permissions for private metadata access in enterprise deployments? Yes. V1 keeps the route stable and isolates field-level redaction so a future private metadata read permission can be added without refactoring routes or persistence.
