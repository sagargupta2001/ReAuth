# Spec: User Invitations

> Distilled from: Invite User system requirements / 2026-05-03
> Status: Ready

---

## User Story

As a realm admin, I want to invite users by email so that accounts can be onboarded through an invitation workflow, tracked separately from active users, and managed with clear lifecycle statuses.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Creates invitations, views invitation status, resends invitations, revokes invitations |
| Invited User | Receives invite email and accepts invitation to create an account |
| Operator | Configures SMTP/public URL; monitors delivery and invitation lifecycle behavior |

---

## Business Rules

1. The Users page must expose two tabs:
   - `All`: existing user list behavior remains unchanged
   - `Invitations`: invitation list rendered with the same DataTable style/pagination/filtering/sorting behavior as `All`
2. Inviting a user creates an invitation record and attempts email delivery through the existing `EmailDeliveryService`.
3. Invited users do not appear in the main Users list until an invitation is accepted and a real `users` record is created.
4. Invitation status is an explicit lifecycle enum shared across backend and UI:
   - `pending`, `accepted`, `expired`, `revoked`
5. Invitation expiration is controlled by `expiry_days` selected in the invite UI and stored per invitation.
6. Resend is allowed only for non-expired `pending` invitations, increments resend metadata, and is capped by a realm-level resend limit.
7. Resend does not extend expiration. If an invitation is expired, admins must create a new invitation.
8. Invitation accept is one-time and must atomically:
   - validate token + realm + status + expiration
   - create user
   - transition invitation to `accepted`
9. Re-inviting the same email is idempotent at the active-invite layer:
   - only one `pending` invitation per `(realm, normalized_email)` is allowed
   - creating a new invite for an expired/revoked/accepted email creates a new row
10. Expiration must be enforced in runtime checks and reflected in list responses; stale `pending` rows are transitioned to `expired` before read/write operations that depend on status.
11. Invitation acceptance must execute through a dedicated realm flow binding (`invitation_flow_id`) so admins can customize invite onboarding nodes like other flows.
12. On successful invitation acceptance and account creation, the user is redirected to the login page and must authenticate normally (no automatic login session creation).

**Edge cases:**
- Admin resends an invite exactly at expiry boundary.
- Same invite accept token submitted concurrently.
- Same email invited multiple times across history.
- Email sending fails after invitation row is created.

---

## Domain Changes

### New Entities

```text
Invitation
  - id: uuid
  - realm_id: uuid
  - email: string
  - email_normalized: string
  - status: InvitationStatus
  - token_hash: string
  - expiry_days: i64
  - expires_at: datetime
  - invited_by_user_id: uuid?
  - accepted_user_id: uuid?
  - accepted_at: datetime?
  - revoked_at: datetime?
  - resend_count: i64
  - last_sent_at: datetime?
  - created_at: datetime
  - updated_at: datetime
```

### New Value Objects / Enums

```text
InvitationStatus (backend enum)
  - Pending
  - Accepted
  - Expired
  - Revoked

InvitationStatus (UI string enum / union)
  - 'pending' | 'accepted' | 'expired' | 'revoked'
```

### Modified Entities

```text
User
  ~ no schema change for invitation feature itself
  ~ created only when invitation is accepted

Realm
  + invitation_flow_id: string? - realm-bound flow for invitation acceptance
  + invitation_resend_limit: i64 - max resends allowed per invitation in this realm
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `src/domain/...` | Add `invitation.rs` with typed `Invitation` and `InvitationStatus` enum (`serde(rename_all = "lowercase")`, `Display`, `FromStr`, SQLx encode/decode pattern similar to `SessionStatus`) |
| `src/ports/...` | Add `InvitationRepository` port and methods for create/list/find-by-token-hash/resend/update-status/expire-pending |
| `src/application/...` | Add `InvitationService` for orchestration: create, list, resend, revoke, accept, expiration transition, idempotency policy |
| `src/adapters/persistence/...` | Add `sqlite_invitation_repository.rs` with indexed/filterable queries and transactional state transitions |
| `src/adapters/web/...` | Add `invitation_handler.rs` + router wiring under `/api/realms/{realm}/invitations` |
| `src/application/email_delivery_service.rs` | Add `send_invitation_email(...)` with invite-specific templates/placeholders and link generation |
| `src/application/flow_*` and flow storage | Add invitation flow type wiring + default invitation flow seed/deployment |
| `src/domain/realm.rs`, `realm_service`, `realm_repository` | Add `invitation_flow_id` + `invitation_resend_limit` fields and update payload handling |
| `ui/src/entities/...` | Add invitation model types and status constants/mappers |
| `ui/src/features/invitation/...` | Add API hooks, columns, table, tab actions, resend/revoke mutations |
| `ui/src/pages/user/listing/UsersPage.tsx` | Add URL-driven tab state (`all`/`invitations`) and render corresponding table |
| `ui/src/features/user/components/CreateUserDialog.tsx` | Replace invite TODO with real invite mutation and error mapping |
| `ui/src/features/realm/forms/GeneralSettingsForm.tsx` | Add invitation resend limit input under Registration card |

---

## Persistence Changes

### New Migration(s)

```text
20260503HHMMSS_add_invitations_table.sql
20260503HHMMSS_add_realm_invitation_settings.sql
```

`add_realm_invitation_settings` adds:
- `realms.invitation_flow_id TEXT NULL` (same no-FK strategy as other realm flow bindings)
- `realms.invitation_resend_limit INTEGER NOT NULL DEFAULT 3`

### Invitations Table Schema

```sql
CREATE TABLE invitations (
  id TEXT PRIMARY KEY NOT NULL,
  realm_id TEXT NOT NULL,
  email TEXT NOT NULL,
  email_normalized TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending','accepted','expired','revoked')),
  token_hash TEXT NOT NULL,
  expiry_days INTEGER NOT NULL,
  expires_at DATETIME NOT NULL,
  invited_by_user_id TEXT,
  accepted_user_id TEXT,
  accepted_at DATETIME,
  revoked_at DATETIME,
  resend_count INTEGER NOT NULL DEFAULT 0,
  last_sent_at DATETIME,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (realm_id) REFERENCES realms(id) ON DELETE CASCADE,
  FOREIGN KEY (invited_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
  FOREIGN KEY (accepted_user_id) REFERENCES users(id) ON DELETE SET NULL
);
```

### Indexing

```sql
CREATE INDEX idx_invitations_realm_created_at ON invitations(realm_id, created_at DESC);
CREATE INDEX idx_invitations_realm_status_created_at ON invitations(realm_id, status, created_at DESC);
CREATE INDEX idx_invitations_realm_email_normalized ON invitations(realm_id, email_normalized);
CREATE INDEX idx_invitations_realm_expires_at ON invitations(realm_id, expires_at);
CREATE UNIQUE INDEX idx_invitations_token_hash ON invitations(token_hash);
CREATE UNIQUE INDEX idx_invitations_pending_email_unique
  ON invitations(realm_id, email_normalized)
  WHERE status = 'pending';
```

### Status Handling Strategy

- Status is stored as lifecycle state and transitioned explicitly.
- Before list/create/resend/accept, repository runs:
  - `UPDATE invitations SET status='expired', updated_at=? WHERE realm_id=? AND status='pending' AND expires_at<=?`
- This keeps DB and API aligned without requiring external infrastructure.

---

## API Changes

### Backend Status Enum Contract

- API status fields always use lowercase string enum values:
  - `pending`, `accepted`, `expired`, `revoked`
- No raw freeform status strings are accepted.

### Endpoints

```text
POST /api/realms/{realm}/invitations
  Request:
    {
      "email": "user@example.com",
      "expiry_days": 7
    }
  Response: 201
    {
      "id": "uuid",
      "email": "user@example.com",
      "status": "pending",
      "expiry_days": 7,
      "expires_at": "2026-05-10T12:00:00Z",
      "resend_count": 0,
      "last_sent_at": "2026-05-03T12:00:00Z",
      "created_at": "...",
      "updated_at": "..."
    }
  Auth: protected, `user:write`
```

```text
GET /api/realms/{realm}/invitations?page=1&per_page=10&q=email&status=pending&sort_by=created_at&sort_dir=desc
  Response: 200 paginated PageResponse<Invitation>
  Auth: protected, `user:read`
  Notes:
    - Supports same pagination/query conventions as users table
    - Status filter values: pending|accepted|expired|revoked
```

```text
POST /api/realms/{realm}/invitations/{id}/resend
  Request: {}
  Response: 200 updated invitation row
  Auth: protected, `user:write`
  Rules:
    - only pending + not expired
    - `resend_count < realm.invitation_resend_limit`
    - resend_count += 1
    - last_sent_at = now
    - expires_at unchanged
```

```text
POST /api/realms/{realm}/invitations/{id}/revoke
  Request: {}
  Response: 200 updated invitation row (status=revoked)
  Auth: protected, `user:write`
  Rules:
    - allowed from pending only
```

```text
POST /api/realms/{realm}/invitations/accept
  Request:
    {
      "token": "opaque_invite_token",
      "username": "newuser",
      "password": "secret"
    }
  Response: 201
    {
      "status": "redirect",
      "url": "/#/login?realm={realm}&invited=1"
    }
  Auth: public
  Rules:
    - token is one-time, realm-scoped, expiration-checked
    - executes configured invitation flow, creates user, marks invitation accepted
    - does not mint refresh/login session; always redirects to login
```

```text
PUT /api/realms/{id}
  Added to request:
    {
      "invitation_resend_limit"?: number,
      "invitation_flow_id"?: string | null
    }
  Response:
    updated realm payload including new fields
  Auth: protected, `realm:write`
```

### Error Semantics

- `422` field validation for malformed email/expiry/input.
- `409` for duplicate active invitation or username/email conflicts on accept.
- `400` for invalid/expired/revoked/consumed token on accept.
- `400` for resend attempts above configured realm limit.
- Security-safe error messages for public accept endpoint (avoid account enumeration).

---

## Flow / Auth Impact

- Flow types affected: registration + new `invitation` flow type
- New realm binding: `invitation_flow_id`
- Invitation acceptance uses the flow engine so admins can customize the onboarding path with nodes/branches.
- Default invitation flow is seeded and deployed per realm, with a minimal safe baseline:
  - validate invite token
  - create account from invite payload
  - terminal success with redirect-to-login
- Async pause/resume impact: invitation token validation is explicit; no background async loop required beyond existing invitation expiry handling.
- Theme/Fluid impact: optional dedicated invite acceptance page key can be added later; initial slice may reuse registration-like templates.

---

## Frontend Architecture

### Users Page Tab Integration

- Keep route `/:realm/users`; use query param `tab` with enum codec:
  - `tab=all` (default)
  - `tab=invitations`
- `UsersPage` renders tab header and switches between:
  - existing `UsersTable`
  - new `InvitationsTable`

### Table Reuse Strategy

- Reuse shared `DataTable` and `DataTableSkeleton`.
- Invitations table follows existing table patterns:
  - URL-synced page/per_page/sort/q
  - React Query + `keepPreviousData`
  - same toolbar slots and action patterns

### State Management

- Use React Query feature hooks:
  - `useInvitations`
  - `useCreateInvitation`
  - `useResendInvitation`
  - `useRevokeInvitation`
- Query keys in `queryKeys`:
  - `invitations(realm, params)`
- Mutations invalidate invitations list and optionally users list on accept-related views.

### Status + Actions UI

- Add shared invitation status mapping (label + badge variant) from typed enum values.
- Row actions:
  - `Resend` visible only for `pending`
  - `Revoke` visible only for `pending`
- Columns:
  - Email
  - Status badge
  - Expires
  - Created
  - Last sent
  - Resend count

### Reuse of Existing Dialog

- `CreateUserDialog` keeps two tabs (`Create user`, `Invite user`).
- Replace invite tab TODO with real submission to `useCreateInvitation`.
- Keep existing expiry-days input and validation, now wired to API errors.

### Realm Settings Integration

- In `Realm Settings -> General -> Registration`, add numeric input:
  - `Invitation Resend Limit` (per realm)
- Validation:
  - integer, min `0`
  - `0` means no manual resend allowed after first send
- Stored in `realm.invitation_resend_limit` and enforced by resend endpoint.

---

## Email Flow

### Integration Strategy

- Extend `EmailDeliveryService` with `send_invitation_email(...)`.
- Reuse existing SMTP realm settings resolution and resume-link construction pattern.

### Token Generation & Validation

- Generate opaque random token (base64url) on create.
- Persist only SHA-256 `token_hash` (never raw token).
- Email contains raw token in invitation accept URL.
- Accept endpoint hashes incoming token and resolves invitation by hash.

### Template Variables

- Subject/body template support (default templates initially):
  - `{realm}`
  - `{email}`
  - `{invite_url}`
  - `{expires_at}`

### Security Considerations

- One-time token: accepted invitation cannot be reused.
- Realm-scoped token enforcement.
- Expiration check at accept and resend.
- Avoid returning whether an email is already registered on public endpoints.
- Audit events for create/resend/revoke/accept/failure.

---

## Expiration Strategy

1. `expires_at = created_at + expiry_days`.
2. Stored status transitions from `pending` -> `expired` via runtime transition query before relevant operations.
3. Resend does not change `expires_at`; it only refreshes delivery metadata.
4. Accept fails when invitation is expired/revoked/already accepted.
5. Expired invitations remain queryable under `expired` for admin traceability.
6. Resend is blocked when `resend_count` reaches `realm.invitation_resend_limit`.

**Edge cases:**
- Resend at exact expiry timestamp: treated as expired (non-resendable).
- Accept race: transaction ensures one request succeeds; subsequent request gets invalid/consumed state.
- Multiple historical invites for same email: only one active pending invite allowed; historical accepted/expired/revoked rows retained.

---

## Scalability Considerations

1. Indexed list query path (`realm_id + status + created_at`) supports large invitation volumes.
2. Active-invite uniqueness enforced by partial unique index for idempotency at DB level.
3. List/filter pagination mirrors existing `PageRequest` and clamps page size in repository.
4. Runtime expiration update is realm-scoped and bounded to rows matching `pending AND expires_at <= now`; avoids full-table scans with index support.
5. Token lookup uses unique hash index (`token_hash`) for O(log n) accept validation.
6. Idempotent create semantics:
   - if pending invite exists for same normalized email, return conflict with actionable error code
   - UI can surface "already invited" + allow resend action path
7. Resend limit check is O(1) on row read (`resend_count`) + realm setting read.

---

## Test Scenarios

1. **Create invitation happy path**
   - Given: admin with `user:write`, valid email, valid expiry days
   - When: admin submits invite
   - Then: invitation row is created as `pending`, email delivery attempted, row appears in Invitations tab

2. **Duplicate active invite**
   - Given: existing pending invite for same realm/email
   - When: admin submits another invite
   - Then: API returns conflict and does not create a second pending row

3. **Resend behavior**
   - Given: pending invite not expired
   - When: admin clicks resend
   - Then: resend_count increments, last_sent_at updates, expires_at unchanged

4. **Expired invite handling**
   - Given: pending invite past expires_at
   - When: admin lists invitations or attempts resend
   - Then: invite is transitioned/reflected as `expired`; resend is rejected

5. **Accept invitation**
   - Given: valid pending invite token
   - When: invited user accepts with valid account payload
   - Then: user is created through invitation flow, invitation becomes `accepted`, token cannot be reused, and response redirects to login

6. **Accept replay / invalid token**
   - Given: accepted/expired/revoked token
   - When: token is submitted again
   - Then: request fails safely and no user is created

7. **UI tab isolation**
   - Given: existing active users and pending invitations
   - When: user switches tabs
   - Then: `All` shows only real users, `Invitations` shows only invites with matching table UX

8. **Resend limit enforcement**
   - Given: invitation with `resend_count == invitation_resend_limit`
   - When: admin clicks resend
   - Then: API rejects resend and UI shows a clear limit-reached message

9. **Flow customization**
   - Given: admin changes `invitation_flow_id` to a custom deployed invitation flow
   - When: invited user accepts
   - Then: execution follows custom flow nodes and still ends in login redirect on success

---

## Out of Scope

- Bulk invitation upload/import.
- Custom invitation email editor UI in this slice.
- Per-invitation role assignment at invite time.
- Invitation analytics dashboards beyond table metadata.

---

## Open Questions

- None. Resolved on 2026-05-03:
  - Invitation acceptance redirects to login (no auto-login session).
  - Invitation resend limit is configurable per realm in General -> Registration.
  - Invitation acceptance is flow-backed via realm-configurable `invitation_flow_id`.
