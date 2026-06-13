# Spec: Session Management Console

> Distilled from: "revamp the Active Sessions page" requirement discussion, 2026-06-13
> Status: Implemented

---

## User Story

As a realm admin / security operator, I want a production-grade Active Sessions console with global and bulk revocation, forced re-authentication, and rich per-session context, so that I can respond to account compromise and audit who is logged in without clicking trash icons one row at a time.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Lists, inspects, bulk-revokes, and forces re-auth on sessions across all users in the realm |
| Security Operator | Same surface as admin; uses bulk/global revoke during incident response |
| End User | Indirectly affected — a revoked session can no longer refresh; a step-up session is forced back through the realm login flow on next refresh |
| Operator | None (no new system/operator prerequisites; see Out of Scope re: GeoIP) |

---

## Context & Grounding

This is **not** a greenfield session model. The page already exists and must be refactored against ReAuth's real data model:

- A "session" is a row in the `refresh_tokens` table. There is no separate session entity.
- "Active" = `revoked_at IS NULL AND replaced_by IS NULL AND expires_at > now` (see `sqlite_session_repository::list`).
- Tokens **rotate on every refresh**: each refresh marks the old token `replaced_by` and inserts a new row in the same `family_id`. The newest active row in a family is the live session.
- The JWT `sid` claim **is** the current refresh-token id (`token_service.rs`: "Session ID (the Refresh Token ID)"). The UI already uses `sid` to detect the current session — this stays.
- `created_at` ≈ token `iat`, `expires_at` ≈ token `exp`, `last_used_at` = last refresh.
- `client_id IS NULL` = root/SSO browser session (Admin Console / cookie-based); non-null = an OAuth client session.
- The repo already has `revoke_all_for_user`, `revoke_family`, `revoke_by_user_and_client`, and `delete_by_id`.
- Permission constants `session:read` and `session:revoke` **already exist** in `domain/permissions.rs` but are unused — the routes currently guard on `user:write`. This spec adopts the dedicated permissions.

Decisions taken for this slice:
- **Audience: admin-only.** The page continues to list all sessions in the realm. End-user self-service ("my devices") is out of scope.
- **GeoIP: dropped.** Show IP address only. No bundled lookup DB (keeps the single-binary stance).
- **Step-up / forced re-authentication: in scope** (designed below against the rotate-on-refresh model).

---

## Business Rules

Numbered, independently testable.

**Listing & context**
1. `GET .../sessions` returns only active sessions for the realm, paginated, newest first — unchanged filter semantics.
2. The serialized session includes `step_up_at` so the UI can show a "Re-auth pending" state.
3. The current session (row id == caller's `sid`) is labelled `Current` and cannot be revoked or step-up'd from this surface.
4. Session **type** is derived (client-side) from `client_id`: null ⇒ `Browser` (Admin Console / SSO root); non-null ⇒ `OAuth Client` (show the client id).
5. Device/OS/browser is derived (client-side) by parsing `user_agent`; unparseable or missing UA renders as `Unknown device`.
6. Status is derived (client-side): `Current` > `Re-auth pending` (step_up_at set) > `Expiring soon` (expires within a fixed threshold) > `Idle` (no use within a fixed threshold) > `Active`.

**Single-session actions**
7. Revoke a single session deletes that refresh-token row (existing `logout` behavior) and emits an audit event.
8. Force re-authentication ("step-up") sets `step_up_at = now` on the target session **without deleting it**, and emits an audit event.

**Bulk & global actions**
9. Bulk revoke accepts an explicit list of session ids and revokes all of them in one request, atomically (all-or-nothing within a transaction), returning the count revoked.
10. "Revoke all other sessions" revokes every active session belonging to the **caller** except the caller's current `sid`. The caller's current id is taken from the caller's validated token, never trusted from the request body.
11. "Revoke user's entire account sessions" revokes every active session for a given `user_id` in the realm (wraps `revoke_all_for_user`), requires **both `session:revoke` and `user:write`**, and emits an audit event.
12. Every revoke/step-up variant validates that targeted sessions belong to the path realm; sessions in other realms are never touched.
13. A bulk request that includes the caller's current `sid` excludes it from revocation (the caller is never logged out by a bulk/global action) — except the explicit single-session revoke of a non-current session.

**Step-up enforcement**
14. While `step_up_at` is set on the live token of a family, **silent refresh is rejected** with a new `ReauthRequired` error (HTTP 401, distinguishable code), forcing the client to start an interactive login/reauth flow. This refresh-time check is always on (the baseline). When the opt-in flag `security.immediate_step_up_invalidation` is `true`, `verify_session` additionally rejects any access token whose `iat < step_up_at`, forcing step-up on the next request without waiting for refresh.
15. Whether the forced re-auth prompts for MFA is decided entirely by the realm's configured login flow composition — this feature does not introduce a new MFA mechanism, it only forces the session back through the existing flow (per core mental model: flows decide what is experienced).
16. Completing a fresh interactive authentication mints a new session/family with `step_up_at` cleared; the access token issued before `step_up_at` is treated as stale on its next validation.

**Edge cases:**
- Revoking an already-revoked/rotated session is idempotent — returns success/no-op, never a hard error (UI already treats "not found / invalid" as a sync update).
- Bulk revoke with an empty id list is a 400 (nothing to do) — distinct from "revoke all others".
- "Revoke all others" when the caller has only the current session revokes nothing and returns count 0.
- Step-up on the caller's own current session is rejected (rule 3) — you cannot force-reauth the session you are using from this surface.
- A session whose `expires_at` has already passed is never returned by the list and cannot be targeted.

---

## Domain Changes

### Modified Entities

```text
RefreshToken (domain/session.rs)
  + step_up_at: Option<DateTime<Utc>> — when set, the live token of the family must re-authenticate; silent refresh is rejected until a fresh interactive auth clears it
```

`FromRow` mapping and the round-trip/serialize tests are updated for the new column.

### New Error Variant

```text
Error::ReauthRequired — maps to HTTP 401 with a stable error code (e.g. "reauth_required") so clients can route the user into an interactive login flow instead of treating it as a generic auth failure
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/session.rs` | Add `step_up_at` to `RefreshToken` + `FromRow`; update tests |
| `domain/permissions.rs` | None (constants `session:read` / `session:revoke` already exist) |
| `error.rs` | Add `Error::ReauthRequired` + status/code mapping (401) |
| `config.rs` / `config/default.toml` | Add `[security] immediate_step_up_invalidation: bool` (default `false`) consumed by `verify_session` |
| `ports/session_repository.rs` | Add `revoke_many`, `revoke_others_for_user` (or list-active-ids helper), `request_step_up`; existing `revoke_all_for_user` reused |
| `adapters/persistence/sqlite_session_repository.rs` | Implement new methods (transactional bulk), include `step_up_at` in `list` SELECT, enforce realm scoping |
| `application/auth_service/mod.rs` | New service methods: `revoke_sessions`, `revoke_other_sessions`, `revoke_user_sessions`, `request_step_up`; enforce `step_up_at` in the refresh path → `ReauthRequired` |
| `adapters/web/session_handler.rs` | New `revoke_sessions_handler` (tagged payload) and `step_up_session_handler`; extract caller `sid`/`user_id` from validated token; emit audit events |
| `adapters/web/router.rs` | New routes; switch session routes from `user:write` to `session:read` (GET) and `session:revoke` (mutations) |
| `bootstrap/seed` | Backfill `session:read` + `session:revoke` onto `super_admin` (mirrors the `user:lock`/`user:ban` backfill pattern) |
| `ui/src/entities/session/model/types.ts` | Add `step_up_at?`; add derived-status / session-type helper types |
| `ui/src/features/session/` | UA-parsing util, session-type + status derivation, action bar (search + "Revoke all other sessions"), checkbox column + bulk toolbar, ellipsis row menu, details drawer |
| `ui/src/features/session/api/useSessions.ts` | Add `useRevokeSessions` (bulk), `useRevokeOtherSessions`, `useRevokeUserSessions`, `useStepUpSession` |
| `ui/src/pages/session/listing/SessionsPage.tsx` | Compose the new action bar + table + drawer |

UI primitives already present and reused: `sheet` (drawer), `dropdown-menu` (row `…` menu), `checkbox` (selection), `alert-dialog`/`confirm-dialog` (destructive confirms). `DataTable` already supports `enableRowSelection`.

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_refresh_tokens_step_up.sql — ALTER TABLE refresh_tokens ADD COLUMN step_up_at DATETIME NULL;
```

### Data Notes

- Forward-only, additive, nullable column. Existing rows default to NULL (no step-up pending). No backfill required.
- Seeding adds `session:read` and `session:revoke` to `super_admin` and backfills them onto an existing `super_admin` role when present.
- No new index required: bulk/all-others queries filter on `realm_id` + `user_id`, both already used by existing queries.

---

## API Changes

### Modified Endpoints

```text
GET /api/realms/{realm}/sessions
  Auth:     changed: session:read  (was user:write)
  Response: adds step_up_at: string|null to each session object  (otherwise unchanged)

DELETE /api/realms/{realm}/sessions/{id}
  Auth:     changed: session:revoke  (was user:write)
  Behavior: unchanged (single revoke)
```

### New HTTP Endpoints

```text
POST /api/realms/{realm}/sessions/revoke
  Request (tagged by scope):
    { "scope": "selected", "session_ids": uuid[] }    // bulk checkbox revoke (non-empty)
    { "scope": "others" }                              // revoke all of caller's sessions except current sid
    { "scope": "user", "user_id": uuid }               // revoke a user's entire account sessions
  Response: { "count": number }
  Auth:     protected, session:revoke  (scope:"user" additionally requires user:write)
  Notes:    caller's current sid is excluded from "selected" and "others"; realm-scoped; transactional

POST /api/realms/{realm}/sessions/{id}/step-up
  Request:  {}  (no body)
  Response: 204 No Content
  Auth:     protected, session:revoke
  Notes:    sets step_up_at=now; rejected for the caller's current session
```

Audit events emitted (via `audit_service.record`, target_type `"session"`):
`session.revoke`, `session.revoke_bulk` (metadata: count), `session.revoke_others` (metadata: count), `session.revoke_user` (metadata: user_id, count), `session.step_up`.

---

## Flow / Auth Impact

- Flow types affected: **browser login / reauth** (enforcement only; no new nodes).
- New nodes: none.
- Existing nodes modified: none.
- Refresh path: `auth_service` rejects silent refresh with `ReauthRequired` while `step_up_at` is set, forcing an interactive run of the realm's login flow.
- Async pause/resume impact: none.
- Theme/Fluid page impact: none (re-auth reuses the realm's existing login pages).

---

## Availability / Admin UX

- System/operator prerequisites: optional `[security] immediate_step_up_invalidation` flag (default `false`) enabling immediate access-token rejection on step-up. (GeoIP intentionally dropped — IP shown verbatim.)
- Realm policy: none new. Step-up reuses the realm's existing login flow composition to decide whether MFA is prompted.
- Flow composition: forced re-auth replays the realm browser login flow; MFA presence is whatever that flow already defines.
- Builder behavior: none.
- Simple mode UX: action bar exposes search + "Revoke all other sessions"; row `…` menu exposes Revoke / Force re-authentication / View JSON / Revoke user's account sessions.
- Advanced mode UX: details drawer shows iat (`created_at`), exp (`expires_at`), last used, token `family_id`, derived type/status, and raw JSON (copyable).

### UI Layout

```
[ 🔍 Search… ]                              [ 🚫 Revoke all other sessions ]
[ ☑ 3 selected → Revoke selected ]          (bulk toolbar appears on selection)

☐ | User (avatar + truncated id, Current badge)
  | Type        Browser • Chrome / macOS
  | Location    127.0.0.1            (IP only)
  | Status      Current / Re-auth pending / Expiring soon / Idle
  | Started     relative time
  | …  → Terminate session · Force re-authentication · View token JSON · Revoke user's account sessions
```

Clicking a row opens the details drawer.

---

## Test Scenarios

1. **Happy path — bulk revoke (selected)**
   - Given: 5 active sessions, 3 selected (none is caller's current).
   - When: `POST .../sessions/revoke { scope:"selected", session_ids:[3] }`.
   - Then: 200 `{ count: 3 }`; those 3 no longer listed; list refetch shows 2; audit `session.revoke_bulk` count=3.

2. **Revoke all other sessions excludes current**
   - Given: caller has 3 active sessions including current `sid`.
   - When: `POST .../sessions/revoke { scope:"others" }`.
   - Then: `{ count: 2 }`; caller's current session still active and usable.

3. **Revoke user's entire account sessions**
   - Given: target user has 4 active sessions across 2 clients.
   - When: `POST .../sessions/revoke { scope:"user", user_id }`.
   - Then: `{ count: 4 }`; none of that user's sessions remain active; audit `session.revoke_user`.

4. **Step-up forces re-auth on next refresh**
   - Given: a non-current session with a valid refresh token.
   - When: `POST .../sessions/{id}/step-up`, then that client attempts a silent refresh.
   - Then: step-up returns 204; refresh is rejected with `ReauthRequired` (401, code `reauth_required`); after a fresh interactive login a new family is minted with `step_up_at` cleared and refresh succeeds.

5. **Permission enforcement**
   - Given: a token with `user:write` but neither `session:read` nor `session:revoke`.
   - When: GET sessions / POST revoke.
   - Then: 403 on both; a token with `session:read` can list but cannot revoke; `session:revoke` can revoke.

6. **Realm scoping**
   - Given: a session id that belongs to realm B.
   - When: targeted via realm A's revoke/step-up endpoint.
   - Then: it is not revoked/modified (treated as not-in-realm); no cross-realm leakage.

7. **Cannot self-revoke / self-step-up current session**
   - Given: caller's current `sid`.
   - When: single revoke or step-up of that id, or its inclusion in a bulk "selected" list.
   - Then: current session is never killed by bulk/global; explicit step-up of current session is rejected; UI disables the controls.

8. **Idempotent revoke of stale session**
   - Given: a session already rotated/revoked.
   - When: revoke it.
   - Then: success/no-op (no 500); UI shows a sync info toast and refreshes.

9. **Validation failure — empty bulk list**
   - Given: `{ scope:"selected", session_ids: [] }`.
   - When: posted.
   - Then: 400.

10. **UI derivation**
    - Given: sessions with various `user_agent` / `client_id` / `expires_at` / `last_used_at`.
    - When: rendered.
    - Then: type, OS/browser, and status badges derive per rules 4–6; unparseable UA ⇒ `Unknown device`; `client_id` null ⇒ `Browser`.

---

## Out of Scope

- **End-user self-service "my sessions" page** (this slice is admin-only).
- **GeoIP / IP-to-location enrichment** (dropped — no bundled lookup DB; conflicts with single-binary stance).
- **IP blacklist / WAF integration** — ReAuth has no firewall/WAF layer; there is nothing to enforce a block against. Future, separate capability.
- **Risk engine / "Suspicious" auto-flagging** — no risk-scoring subsystem exists; status badges are deterministic (time/flag based), not risk-scored.
- **Per-session OAuth scope listing in the drawer** — scopes are not persisted on `refresh_tokens`; would require a schema change. Drawer shows token family + timing instead.
- **Background purge of revoked/replaced/expired rows** — already tracked as a separate roadmap item (`auth-production-grade.md`).

---

## Resolved Decisions

All four prior open questions are resolved (2026-06-13):

- [x] **Status thresholds:** `Expiring soon` = `expires_at` within **1h**; `Idle` = no activity (no `last_used_at` update / refresh-token rotation) within a rolling **24h** window. Badge tooltips state the meaning explicitly ("No activity in the last 24 hours") so `Idle` is not misread as compromise. Presentational only, no backend impact.
- [x] **Step-up permission:** reuse `session:revoke` (step-up is a softer, conditional form of revocation; anyone who can destroy a session can conditionally restrict it). The handler gates via a normal permission check so a dedicated `session:step-up` string can be split out later without changing endpoint logic.
- [x] **"Revoke user's account sessions" permission:** requires **both** `session:revoke` AND `user:write`. Mass account eviction crosses from token management into the user/identity lifecycle, so it demands user-write authority — preventing a token-clearing-only operator from globally disrupting an account.
- [x] **Step-up enforcement:** **refresh-time enforcement is the baseline** (rule 14). Add an opt-in global config flag `security.immediate_step_up_invalidation` (default `false`). When `true`, `verify_session` additionally rejects any access token whose `iat < step_up_at` (stateless check, reusing the session lookup already performed), forcing step-up on the next request rather than waiting for refresh.
