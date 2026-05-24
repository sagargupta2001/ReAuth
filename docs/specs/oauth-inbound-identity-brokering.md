# Spec: OAuth Inbound Identity Brokering (Social / Enterprise Login)

> Distilled from: product direction discussion / 2026-05-17
> Status: Implemented

---

## Scope Note: "OAuth" in ReAuth

ReAuth already ships an **outbound** OIDC provider role: third-party apps register as `oidc_clients` and call ReAuth's `/oidc/authorize` + `/oidc/token` to authenticate users (see `docs/memory/04-oidc-sso-flows.md`).

This spec defines the **inbound** counterpart: letting realm admins broker external OAuth 2.0 / OIDC providers (Google, GitHub, Microsoft, Apple, generic OIDC/OAuth2) into ReAuth, so end users can pick "Sign in with Google" on the ReAuth login page and a corresponding ReAuth user is authenticated, provisioned, or linked.

The new domain concept is **Identity Provider (IdP)**. It is intentionally distinct from `oidc_clients` to avoid overloading an unrelated concept.

This aligns with `docs/memory/roadmaps/identity-brokering.md` (Phase 5: Enterprise Identity Brokering).

---

## User Story

As a realm admin, I want to configure one or more external OAuth/OIDC identity providers and place a "sign in with provider X" branch into my flows, so that my users can authenticate against Google/GitHub/Microsoft/etc. while ReAuth still owns sessions, RBAC, theming, audit logs, and account lifecycle.

As an end user, I want to sign in with my existing external account, optionally link it to my local ReAuth account, and see a consistent theme and consent UX throughout.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Operator | Configures system prerequisites: public base URL, allowed callback origins, secret encryption key, optional global IdP defaults. |
| Realm Admin | Creates/edits realm-scoped IdP configurations, sets default linking/JIT policy, picks claim mappings, places OAuth branches in flows, customizes theme pages and consent copy. |
| End User | Picks an IdP button on the login page, completes external consent, lands on link/conflict UX when needed, and signs into ReAuth. |
| OIDC Client (downstream) | Receives the resulting ReAuth session/auth code; unaware of which upstream IdP was used. |
| External IdP | Issues authorization code + tokens + userinfo. |

---

## Business Rules

1. Inbound IdP authentication must be a first-class flow node (or pair of nodes) on the existing graph engine — not a script, plugin, or out-of-band route. End-user UX always traverses the configured flow.
2. IdPs are **realm-scoped** by default. A realm with no enabled IdPs sees no change in login UX.
3. Each IdP configuration must support at minimum:
   - protocol kind: `oidc` (auto-discovery via `.well-known`) or `oauth2` (manual endpoints + userinfo URL)
   - client id + client secret (secret is encrypted at rest)
   - scopes (default + admin-editable)
   - issuer / authorization endpoint / token endpoint / userinfo endpoint / jwks endpoint
   - PKCE policy: required (default) | disabled
   - claim mapping (external claim path -> ReAuth user attribute)
   - allowed callback origins (resolved from realm + system base URL)
4. ReAuth must support common providers via built-in **provider presets** (`google`, `github`, `microsoft`, `apple`, `gitlab`, `okta`, `auth0`, `custom-oidc`, `custom-oauth2`). Presets seed sensible defaults but every field must remain admin-editable for maximum customization.
5. Provider buttons rendered on the login page must come from realm IdP config — order, label, icon, color, and visibility are admin-customizable through the same theme/Fluid system used by other auth pages.
6. The OAuth callback must be a single deterministic public endpoint per realm and must:
   - validate state (CSRF) + PKCE verifier
   - require the originating `auth_session` cookie
   - reject if the realm-scoped IdP is disabled or deleted
   - reject if the callback redirect_uri does not match the configured allowed origins
7. After upstream identity is verified, the broker must resolve the user in this strict order, governed by realm policy:
   1. **Existing federation link**: `federated_identities(realm, provider_id, subject)` -> use linked user
   2. **Email match**: if realm allows email auto-link AND email is verified by IdP, link to existing user with matching email
   3. **JIT provisioning**: if realm allows JIT and no match, create a new user from claim mapping
   4. **Manual link**: prompt the user to log in with local credentials and confirm linking
   5. **Reject**: surface a theme-driven "no account" page
8. JIT-provisioned users must follow the same realm policies as native registration: default roles, email-verification policy, password-set policy, and registration-enabled flag (registration may be disabled while IdP login is allowed if JIT is also disabled).
9. Account linking (manual or automatic) must be a one-way explicit action stored as a `federated_identities` row; the same external subject must never link to two users in the same realm.
10. Users must be able to view and unlink federated identities from their account credentials view, subject to a realm "minimum auth factor" guard (cannot unlink the last sign-in method).
11. Every IdP-driven step must emit structured audit events for: `idp_redirect_issued`, `idp_callback_received`, `idp_callback_success`, `idp_callback_failure`, `idp_user_linked`, `idp_user_unlinked`, `idp_jit_provisioned`, `idp_conflict_email_collision`, `idp_state_mismatch`, `idp_pkce_failure`, `idp_token_exchange_failure`, `idp_userinfo_failure`. Events must carry `realm_id`, `provider_id`, `auth_session_id`, and (when known) `user_id`, and must flow through the existing telemetry pipeline so they are queryable from the Logs UI.
12. IdP availability follows ReAuth's three-layer capability model:
    - system/operator: public base URL, callback origin policy, secret encryption configured
    - realm policy: which IdPs are enabled, linking/JIT/email-match defaults, allowed claim mappings
    - flow composition: which flow branches actually show the IdP button (browser login, registration, reauth, invitation acceptance)
13. The flow builder must:
    - expose `core.auth.oauth_idp` as a normal authenticator-styled palette node
    - validate at publish time that the node's referenced provider exists + is enabled
    - block publish if no IdP is enabled and the flow's only auth path is OAuth
    - allow multiple OAuth branches in one flow (one per provider) or one shared "broker" node that chooses provider at runtime via state/cookie
14. Theme/Fluid impact:
    - dedicated system page key(s) for IdP-driven UX (`oauth_select`, `oauth_link_confirm`, `oauth_conflict`, `oauth_redirecting`, `oauth_failure`)
    - login page exposes a "Provider Buttons" block that resolves at render time from enabled IdPs
    - default page blueprints seeded so the theme editor opens these pages immediately
15. Secrets (client_secret, signing keys, PKCE verifiers, state nonces) must be encrypted at rest using the existing `secret_service` abstraction; raw secrets are never returned by the admin API after creation.
16. State + PKCE verifier are session-scoped, one-time, expiring (default 600s), bound to `auth_session_id` + `realm_id` + `provider_id`, and consumed atomically (single-winner update).

**Edge cases:**

- User has a ReAuth account with email `a@b.com`; IdP returns the same email but `email_verified=false`. Auto-link must not occur; conflict page must offer manual link.
- Two IdPs return the same email for the same realm. Both should be linkable to the same user, but only one federation row per `(realm, provider, subject)`.
- Upstream provider returns `error=access_denied`; callback must end in `oauth_failure` page without breaking the auth session.
- Upstream provider revokes/rotates tokens; ReAuth must not depend on upstream refresh tokens for ongoing session validity — ReAuth issues its own session at the end of the flow.
- Realm admin disables an IdP while a user is mid-flow; callback must reject cleanly.
- Realm admin deletes an IdP that has linked users; deletion must be soft (mark disabled, keep federation rows) unless admin explicitly opts into hard-delete.
- Clock skew on `id_token` `iat`/`exp`/`nonce` validation.

---

## Domain Changes

### New Entities

```text
IdentityProvider
  - id: uuid - primary key
  - realm_id: uuid - owning realm
  - alias: text - URL-safe stable identifier used in callback paths (unique per realm)
  - display_name: text - admin/user-visible label
  - protocol: text - "oidc" | "oauth2"
  - preset_key: text? - preset used at create time (google, github, microsoft, ...); informational
  - enabled: bool - realm-level on/off
  - client_id: text - OAuth client id
  - client_secret_ref: text - encrypted secret reference resolved via secret_service
  - issuer: text? - OIDC issuer URL (required when protocol=oidc)
  - authorization_endpoint: text? - OAuth authorize URL
  - token_endpoint: text? - OAuth token URL
  - userinfo_endpoint: text? - userinfo URL (oauth2 or oidc fallback)
  - jwks_uri: text? - OIDC JWKS URL (cached upstream)
  - end_session_endpoint: text? - OIDC end-session URL
  - scopes_json: text - JSON array of requested scopes
  - claim_mapping_json: text - JSON mapping of external claim path -> reauth attribute
  - pkce_required: bool - default true
  - allow_login: bool - whether IdP can be used to sign in
  - allow_link: bool - whether IdP can be linked to existing accounts
  - allow_jit_provisioning: bool - realm policy override per provider
  - allow_email_auto_link: bool - per-provider override
  - require_verified_email: bool - require email_verified=true before auto-link
  - icon_ref: text? - asset reference / preset slug for button rendering
  - button_color: text? - admin-customized button color
  - sort_order: i64 - rendering order on login page
  - metadata_cached_at: timestamp? - last OIDC discovery refresh
  - metadata_cache_json: text? - cached discovery doc
  - created_at: timestamp
  - updated_at: timestamp

FederatedIdentity
  - id: uuid - primary key
  - realm_id: uuid - owning realm
  - provider_id: uuid - linked IdentityProvider
  - user_id: uuid - linked ReAuth user
  - subject: text - external `sub` (or equivalent stable id)
  - external_username: text? - admin-visible username from claims
  - external_email: text? - email from claims (denormalized for audit)
  - raw_claims_json: text? - last-seen claim payload (size-bounded, optional)
  - linked_via: text - "auto_email" | "manual" | "jit"
  - last_login_at: timestamp?
  - created_at: timestamp
  - updated_at: timestamp

OAuthBrokerState
  - id: uuid - primary key, returned to upstream as `state`
  - realm_id: uuid
  - provider_id: uuid
  - auth_session_id: uuid - active flow session
  - pkce_verifier_hash: text - SHA-256 hash of PKCE verifier (raw verifier never persisted)
  - nonce: text? - OIDC nonce sent upstream
  - redirect_uri: text - exact callback URI sent upstream (must match on return)
  - expires_at: timestamp - default 600s
  - consumed_at: timestamp? - replay-protection marker
  - created_at: timestamp
```

### Modified Entities

```text
Realm
  + idp_broker_enabled: bool - master switch for inbound IdP brokering
  + idp_default_jit_policy: text - "allow" | "deny" | "per_provider"
  + idp_default_email_link_policy: text - "allow_verified" | "manual_only" | "deny"
  + idp_minimum_remaining_factor: bool - guard against unlinking last sign-in method

User
  ~ no schema change required; federation lives in FederatedIdentity
  + (derived) has_federated_identities: bool exposed via admin API
```

### New Value Objects

```text
IdentityProviderPreset - immutable preset definition (alias defaults, scopes, endpoints, claim map, icon)
OAuthBrokerResult - normalized outcome: { user_id, action: "logged_in" | "jit_provisioned" | "linked" | "conflict" | "failed" }
ClaimMapping - resolved mapping object used during JIT provisioning
LinkResolution - { kind: "federation" | "email_auto" | "manual" | "jit" | "reject", reason: text }
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `src/domain/identity_provider.rs` (new) | `IdentityProvider`, `FederatedIdentity`, `OAuthBrokerState` domain types and protocol enum |
| `src/domain/flow/nodes/oauth_idp_node.rs` (new) | `NodeProvider` for `core.auth.oauth_idp` with config schema (provider alias, branch outputs) |
| `src/ports/...` | `IdentityProviderRepository`, `FederatedIdentityRepository`, `OAuthBrokerStateRepository` |
| `src/application/idp_service.rs` (new) | CRUD, metadata refresh, secret encryption, provider preset application |
| `src/application/oauth_broker_service.rs` (new) | Issue authorize URL + state, exchange code, fetch userinfo, validate id_token, resolve linking |
| `src/application/flow_executor` | Recognize `oauth_idp` node: emit `SuspendForUI` with redirect data; resume on callback resolution |
| `src/adapters/auth/oauth_idp_authenticator.rs` (new) | Runtime worker that suspends flow with redirect context and consumes broker resolution |
| `src/adapters/web/oauth_broker_handler.rs` (new) | Public routes: redirect issuance + callback |
| `src/adapters/web/idp_admin_handler.rs` (new) | Admin CRUD endpoints under `/api/realms/{realm}/identity-providers` |
| `src/adapters/web/user_credentials_handler.rs` | Add federated-identity inventory + unlink endpoints |
| `src/adapters/persistence/...` | New SQLite repositories for IdP, federated identity, broker state |
| `src/application/audit_service.rs` + telemetry | New audit event variants and structured fields |
| `src/application/seed*` | Built-in provider presets seed (no per-realm seeds; user opts in) |
| `src/application/flow_manager/templates.rs` | Optional preset templates: browser-flow-with-google, etc. (admin opt-in, not default) |
| `src/application/flow_publish_validator.rs` | Validate `core.auth.oauth_idp` references an enabled provider |
| `ui/src/features/identity-provider/` (new) | List, create-from-preset, edit, enable/disable, test-connection UI |
| `ui/src/features/flow-builder/` | Node palette + inspector for `oauth_idp`; provider picker; preset-flow apply action |
| `ui/src/features/auth/` | Login page provider-buttons block, redirecting page, link/conflict pages |
| `ui/src/features/theme/` + `ui/src/features/fluid/` | New system page blueprints: `oauth_select`, `oauth_link_confirm`, `oauth_conflict`, `oauth_redirecting`, `oauth_failure`; new "Provider Buttons" Fluid block |
| `ui/src/features/user/components/UserCredentialsDialog.tsx` (or equivalent) | Federated-identities tab with unlink action |
| `ui/src/features/realm/forms/` | "Identity Brokering" realm settings card |
| `ui/src/pages/observability/...` | Pre-built log filters for `idp_*` event family |

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_create_identity_providers.sql - per-realm IdP configs
YYYYMMDDHHMMSS_create_federated_identities.sql - user <-> external identity link
YYYYMMDDHHMMSS_create_oauth_broker_state.sql - one-time state + PKCE verifier hashes
YYYYMMDDHHMMSS_add_realm_idp_settings.sql - realm-level brokering policy columns
```

### Data Notes

- All tables are realm-scoped via `realm_id` and indexed appropriately.
- `identity_providers(realm_id, alias)` unique; `alias` is used in callback URL path: `/api/realms/{realm}/auth/oauth/{alias}/callback`.
- `federated_identities(realm_id, provider_id, subject)` unique to prevent split-brain links.
- `federated_identities(realm_id, user_id)` indexed for fast user-credentials lookups.
- `oauth_broker_state(realm_id, expires_at)` indexed for bounded cleanup.
- Broker state must store `pkce_verifier_hash` (SHA-256) and `nonce`, never raw verifier.
- Client secrets are stored encrypted via `secret_service`; admin API never echoes raw secret after creation (returns only `client_secret_set: bool` and a masked tail).
- OIDC discovery doc and JWKS are cached in `metadata_cache_json` + `metadata_cached_at` with a refresh policy (default TTL 24h, manual refresh action available).
- Existing realms default to `idp_broker_enabled = false`.

---

## Concurrency / Scalability Requirements

- State consumption must be atomic single-winner:
  - `UPDATE oauth_broker_state SET consumed_at = now WHERE id = ? AND consumed_at IS NULL AND expires_at > now`.
  - Callback proceeds only when `rows_affected == 1`.
- Federation link creation must enforce `(realm_id, provider_id, subject)` uniqueness at DB level (not app-only).
- JIT user creation must be wrapped in a transaction with federation link insert; on conflict, roll back and surface `idp_conflict_email_collision` audit.
- Metadata refresh (`.well-known` + JWKS) must be cached per provider and refresh under a per-provider lock to avoid stampedes; refresh failures must keep serving the previous cache and emit a warning event.
- `id_token` signature verification must use JWKS with cached keys and configurable algorithm allowlist (default `RS256`, `ES256`).
- Background cleanup loop removes expired/consumed broker state in bounded batches (reuse passkey-challenge cleanup pattern).

---

## API Changes

### Admin: Identity Providers

```text
GET /api/realms/{realm}/identity-providers
  Response: list of IdP rows (secret omitted, metadata cache summary included)
  Auth: protected, realm:read

POST /api/realms/{realm}/identity-providers
  Request: { preset?: string, alias: string, display_name: string, protocol: "oidc" | "oauth2", client_id: string, client_secret?: string, issuer?: string, ...endpoints, scopes, claim_mapping, pkce_required, allow_login, allow_link, allow_jit_provisioning, allow_email_auto_link, require_verified_email, icon_ref?, button_color? }
  Response: 201 IdP row (secret masked)
  Auth: protected, realm:write

GET /api/realms/{realm}/identity-providers/{id}
  Auth: protected, realm:read

PUT /api/realms/{realm}/identity-providers/{id}
  Request: partial update; client_secret only re-sent if rotated
  Auth: protected, realm:write

DELETE /api/realms/{realm}/identity-providers/{id}
  Request: { hard?: boolean }
  Behavior: soft-disable by default; hard delete requires no live federation rows OR explicit `hard=true` with cascade
  Auth: protected, realm:write

POST /api/realms/{realm}/identity-providers/{id}/refresh-metadata
  Response: refreshed metadata cache summary
  Auth: protected, realm:write

POST /api/realms/{realm}/identity-providers/{id}/test-connection
  Behavior: attempt discovery + sample token endpoint reachability; never executes a real user flow
  Auth: protected, realm:write

GET /api/realms/{realm}/identity-providers/presets
  Response: list of built-in presets (no realm state)
  Auth: protected, realm:read
```

### Public: OAuth Broker

```text
GET /api/realms/{realm}/auth/oauth/{alias}/start
  Behavior: requires active auth_session cookie + flow paused on oauth_idp node; issues OAuthBrokerState, returns redirect URL to external IdP
  Response: { redirect_url: string }
  Auth: public (auth_session-bound)

GET /api/realms/{realm}/auth/oauth/{alias}/callback?code=...&state=...
  Behavior: validates state + PKCE, exchanges code, validates id_token, fetches userinfo, resolves linking, resumes flow
  Response: 302 to next flow step OR JSON challenge for /auth/login executor (handler chooses based on Accept)
  Auth: public (state-bound)
```

### User Credentials: Federated Identities

```text
GET /api/realms/{realm}/users/{id}/credentials
  Changed response: now includes `federated_identities: [{ provider_alias, provider_display_name, subject, external_email, linked_via, last_login_at }]`
  Auth: protected, user:read

DELETE /api/realms/{realm}/users/{id}/credentials/federated/{federated_identity_id}
  Behavior: rejects if removing this link would leave the user with no usable sign-in method per realm.idp_minimum_remaining_factor
  Auth: protected, user:write
```

### Realm Settings

```text
PUT /api/realms/{id}
  Added fields:
    {
      "idp_broker_enabled"?: boolean,
      "idp_default_jit_policy"?: "allow" | "deny" | "per_provider",
      "idp_default_email_link_policy"?: "allow_verified" | "manual_only" | "deny",
      "idp_minimum_remaining_factor"?: boolean
    }
  Auth: protected, realm:write
```

### Modified Endpoints

```text
GET /api/realms/{realm}/auth/login
  Changed response context: includes `enabled_providers` array (alias, display_name, icon_ref, button_color, sort_order) for theme rendering of provider buttons

POST /api/realms/{realm}/auth/login/execute
  Added behavior: accepting an `oauth_idp` selection input that progresses the flow to the redirect-issuance node
```

---

## Flow / Auth Impact

- Flow types affected: `browser`, `registration` (post-link onboarding), step-up `reauth` (re-broker), `invitation` (accept-via-IdP).
- New nodes:
  - `core.auth.oauth_idp` — single configurable node that:
    - inputs: optional `provider_alias` (static config) OR runtime input from a preceding `core.auth.collect_idp_choice` node
    - outputs: `logged_in`, `jit_provisioned`, `link_required`, `conflict`, `failed`
    - default_template_key: `oauth_redirecting`
  - `core.auth.collect_idp_choice` (optional) — renders the provider picker page and emits the chosen alias.
- Existing nodes modified:
  - `core.auth.password`: gains an explicit "show provider buttons" rendering hint (handled by theme block, not node config).
  - browser flow template: optionally extended by admin-applied preset (no change to default).
- Async pause/resume impact: flow suspends with `SuspendForUI` carrying redirect data; resumes when callback handler consumes broker state and updates the auth session context. Reuses existing pause/resume infrastructure — no new top-level scheduler.
- Theme/Fluid impact: see "Theme & Fluid Impact" section below.
- Dedicated system pages: `oauth_select`, `oauth_link_confirm`, `oauth_conflict`, `oauth_redirecting`, `oauth_failure`.

---

## Theme & Fluid Impact

- New system page keys (auth category):
  - `oauth_select` — chooses among multiple providers (used when flow uses `collect_idp_choice`)
  - `oauth_redirecting` — interstitial while issuing redirect / on return
  - `oauth_link_confirm` — prompts user to confirm linking IdP identity to existing local account (asks for local password / re-auth)
  - `oauth_conflict` — email collision or unverified email branch
  - `oauth_failure` — provider error / access_denied / state mismatch
- New Fluid block: `ProviderButtons`
  - Renders dynamically from `enabled_providers` context emitted by the executor.
  - Props: layout (`stack` | `grid`), button size, show-icon, show-label-only.
  - Per-provider button color and icon resolved from IdP config.
- Default page blueprints seeded so the theme editor exposes the new pages immediately, matching the convention established by passkeys.
- Existing `login` page blueprint extended with an optional `ProviderButtons` block above the password form.

---

## Availability / Admin UX

- System/operator prerequisites:
  - configured public base URL (already required by recovery + magic links)
  - `secret_service` configured for encryption-at-rest
  - HTTPS in any non-development deployment
- Realm policy:
  - `idp_broker_enabled` master switch
  - default JIT + email-link policies (overrideable per provider)
  - `idp_minimum_remaining_factor` guard for unlink protection
- Flow composition:
  - browser flow can prepend a provider-buttons branch
  - reauth flow can require IdP re-broker for step-up
  - invitation acceptance can offer "accept via Google" as an alternative to setting a password
- Builder behavior:
  - hide/disable `core.auth.oauth_idp` when `idp_broker_enabled=false` or no provider is enabled
  - inspector shows a provider picker drop-down sourced from `/identity-providers`
  - publish fails with actionable error if the referenced provider is missing/disabled
- Simple mode UX:
  - Realm Settings -> "Identity Brokering" card with master switch + default policies
  - Per-provider quick toggle list
  - Preset actions: "Add provider buttons to browser login"
- Advanced mode UX:
  - Flow builder exposes `oauth_idp` and `collect_idp_choice` explicitly
  - Admins can branch on outputs (`conflict` -> custom remediation, `jit_provisioned` -> onboarding subflow, etc.)
- Logs UX:
  - Pre-built filter `category=idp` surfaces the new audit family
  - Each event includes `provider_alias` and (when known) `subject` for triage
- Users UX:
  - Users table gains an optional "Federated" column (provider count) and a filter
  - User detail view shows linked identities with last-login timestamps and an "Unlink" action

---

## Test Scenarios

1. **Provider preset creation**
   - Given: admin opens "Add provider" and picks `google` preset
   - When: admin supplies client_id + client_secret and saves
   - Then: IdP row exists with preset defaults applied, secret stored encrypted, button visible on login page after enable

2. **First-time JIT login**
   - Given: realm with `idp_broker_enabled=true`, Google IdP with `allow_jit_provisioning=true`
   - When: a previously unknown user signs in with Google
   - Then: a new ReAuth user is created using claim mapping, federation row created with `linked_via=jit`, audit `idp_jit_provisioned` written, user is logged in and downstream OIDC client receives a session

3. **Email auto-link (verified)**
   - Given: existing ReAuth user `alice@x.com`, Google IdP with `allow_email_auto_link=true`, `require_verified_email=true`
   - When: Google returns `email=alice@x.com, email_verified=true` for an unlinked subject
   - Then: federation row is created with `linked_via=auto_email`, user logged in, audit `idp_user_linked` written

4. **Email collision (unverified)**
   - Given: existing ReAuth user `alice@x.com`, IdP returns matching email with `email_verified=false`
   - When: callback handler resolves linking
   - Then: flow lands on `oauth_link_confirm` page requiring local credentials before linking; auto-link does not occur

5. **State + PKCE mismatch**
   - Given: tampered or expired state in callback
   - When: callback handler validates state
   - Then: callback fails safely, broker state cannot be reused, audit `idp_state_mismatch` written, user lands on `oauth_failure` page without breaking auth session

6. **Concurrent callback race**
   - Given: same valid state submitted concurrently from two requests
   - When: both callbacks attempt resolution
   - Then: exactly one consumes state and proceeds, the other fails with a generic error and audit entry

7. **Disabled provider mid-flow**
   - Given: user in middle of flow at `oauth_idp` node
   - When: admin disables the provider before callback returns
   - Then: callback is rejected with `idp_callback_failure`, user sees `oauth_failure` page

8. **Unlink last factor protection**
   - Given: user has only federated identity and no password set, realm `idp_minimum_remaining_factor=true`
   - When: user attempts to unlink the federation
   - Then: API returns 409 with a clear message; password set / passkey enroll is suggested

9. **Flow publish validation**
   - Given: flow uses `oauth_idp` referencing a deleted provider
   - When: admin attempts to publish
   - Then: publish fails with provider-resolution error before deployment

10. **Multi-provider login page**
    - Given: realm enables Google + GitHub + Microsoft
    - When: end user opens login page
    - Then: provider buttons render in configured sort order with admin-customized icons/colors; clicking each routes to the correct alias

11. **Audit event observability**
    - Given: end-to-end IdP login that succeeds
    - When: admin opens Logs with `category=idp` filter
    - Then: ordered events `idp_redirect_issued -> idp_callback_received -> idp_callback_success -> idp_user_linked (or idp_jit_provisioned)` are visible with correlated `request_id`/`trace_id`

---

## Out of Scope

- Outbound SAML / SAML 2.0 IdP brokering (separate roadmap item).
- LDAP / Active Directory binding (Phase 5 follow-up, separate spec).
- SCIM provisioning (Phase 5 follow-up).
- IdP-initiated single sign-on (this spec is SP-initiated only).
- Token-exchange grant or RFC8693 (`docs/memory/roadmaps/advanced-iam.md`).
- Per-client IdP routing rules (mentioned in identity-brokering roadmap as `Later`; not included here).
- Native mobile deep-link callback handling (browser callbacks only for this slice).
- Custom claim-mapping DSL beyond a JSON path -> attribute map (a richer mapping/transform language is a follow-up).

---

## Open Questions

All v1 open questions are resolved below. Items deferred to a later slice are listed under "Deferred / Follow-Up".

### Resolved on 2026-05-17

- [x] **`core.auth.oauth_idp` node shape**: Support both modes. Node config `provider_alias` is optional; when a preceding `core.auth.collect_idp_choice` node emits a runtime alias into the session context, runtime input wins. This gives admins a one-node-per-provider layout when they want explicit branches per provider, and a single-broker layout when they want one picker page to fan out.
  - Why: matches ReAuth's "explicit flow composition over magic" stance while keeping the simple case ergonomic. Both shapes use the same runtime worker.

- [x] **IdP rate limiting**: Ship a dedicated per-realm + per-IdP throttle on the broker `/start` endpoint, layered on top of global request middleware. Default: 30 starts per IP per provider per 10 minutes, realm-configurable on `RealmIdpSettings` (reuse the recovery rate-limit pattern).
  - Why: identity brokering is a known account-takeover surface (mass JIT enumeration / IdP token-exchange abuse). Global middleware alone does not give per-IdP visibility, and per-IdP limits make abuse audit events actionable.

- [x] **Canonical email claim**: Use OIDC `email` + `email_verified` by default. Each preset declares a `claim_fetch_strategy`:
  - `id_token` (default): read from `id_token` claims merged with `/userinfo`.
  - `endpoint`: for providers like GitHub that require a secondary call (e.g., `/user/emails` for primary email), the preset wires a typed fetcher that returns a normalized `email` + `email_verified` pair before claim mapping runs.
  - Why: keeps the per-provider quirks isolated to preset code paths, while the claim-mapping JSON stays uniform and admin-editable.

- [x] **JIT email verification trust**: Trust IdP `email_verified=true` only when the provider config has `require_verified_email=true`. If `require_verified_email=false`, JIT-provisioned users land on the realm's standard email-verification path (same as native registration). Email-auto-link to an existing user always requires `email_verified=true` regardless of this flag.
  - Why: the conservative default avoids account-takeover via providers that mark unverified emails. Operators who trust their IdP can opt in per provider.

- [x] **Federation deletion ownership**: Self-service by default for the end user, subject to the `idp_minimum_remaining_factor` realm guard. Admins can also delete federation rows from the user detail view. No separate "self-service opt-in" flag is needed.
  - Why: mirrors how passkeys and password credentials are already managed in the user-credentials view. The minimum-factor guard already prevents lock-out.

- [x] **Broker flow type vs branch**: Branch-only inside existing flow types (`browser`, `registration`, `reauth`, `invitation`). No new `auth_oauth_broker` flow type.
  - Why: ReAuth's flow catalog deliberately keeps flow types small and binds them to realm slots. Brokering is a UX branch, not a new journey kind; making it a branch keeps it composable with password, passkey, and magic-link branches.

- [x] **Discovery / JWKS cache storage**: Inline columns on `identity_providers` (`metadata_cache_json`, `metadata_cached_at`, plus a sibling `jwks_cache_json` + `jwks_cached_at`). No separate cache table in v1.
  - Why: SQLite-friendly, no extra schema, no cross-table joins on the hot callback path. If JWKS caching grows to need TTL-per-key or multi-version retention, a split table is a forward-only migration.

- [x] **Auto-link policy preview UI**: Out of scope for v1. The `test-connection` admin action covers reachability; claim-mapping debugging is supported via the new `category=idp` log family showing `idp_callback_received` with the raw mapped attributes during a real test sign-in.
  - Why: real callback events plus existing log filtering already give admins what they need; a simulator adds surface area without a clear v1 win.

- [x] **ReAuth-realm as upstream IdP for another ReAuth realm**: Out of scope for v1 but explicitly **not blocked**. A ReAuth realm exposes a standard OIDC discovery doc, so another ReAuth deployment configures it as a generic `custom-oidc` preset today. No bespoke domain model changes are required.
  - Why: defers special-casing until at least one production user asks for cross-realm/cross-deployment SSO. The generic OIDC path covers it.

### Deferred / Follow-Up (post-v1)

- Per-provider auto-link policy simulator UI.
- Rich claim-mapping DSL beyond JSON-path → attribute (transforms, conditionals, group/role assignment from claims).
- Cross-realm broker UX presets for "this ReAuth deployment trusting another ReAuth realm".
- Per-client IdP routing rules (mentioned as `Later` in the identity-brokering roadmap).

---

## Implementation Phases (Suggested)

### Phase 1: Foundation
- Domain types, repositories, migrations.
- `idp_service` CRUD + preset list + secret encryption.
- Admin endpoints + Identity Brokering realm settings card.

### Phase 2: OAuth Broker Runtime
- `oauth_broker_service` (authorize URL issuance, state, PKCE, code exchange, id_token + userinfo verification).
- Public `/auth/oauth/{alias}/start` and `/auth/oauth/{alias}/callback` endpoints.
- `core.auth.oauth_idp` node + runtime worker.
- Audit events + telemetry wiring.

### Phase 3: Login UX
- Provider buttons block on login page.
- `oauth_redirecting` + `oauth_failure` default pages.
- Browser-flow preset action: "Add provider buttons".

### Phase 4: Linking + Conflict UX
- `oauth_link_confirm` + `oauth_conflict` pages.
- Federation row CRUD + user credentials inventory + unlink endpoint.
- Realm `idp_minimum_remaining_factor` guard.

### Phase 5: JIT + Customization
- JIT provisioning + claim mapping resolver.
- `collect_idp_choice` node + `oauth_select` page (for branched flows).
- Builder publish validation + provider picker UX.

### Phase 6: Hardening
- Metadata cache refresh policy + manual refresh action.
- Broker-state cleanup loop.
- Rate-limit policy + abuse audit events.
- Rollout runbook + provider-specific notes (Google, GitHub, Microsoft, Apple, generic OIDC, generic OAuth2).
  - Implemented in `docs/memory/21-identity-brokering-operations.md`.

---

## Rollout Runbook

Canonical rollout and provider-operations guidance lives in:

- `docs/memory/21-identity-brokering-operations.md`

---

## Decisions (Locked)

1. IdPs are realm-scoped first-class entities, separate from `oidc_clients`.
2. Inbound brokering executes through the flow engine via a new `core.auth.oauth_idp` node; no out-of-flow shortcut routes.
3. Secrets are stored encrypted via `secret_service`; admin API never echoes raw values after creation.
4. Default linking order: federation row -> email auto-link (if verified) -> manual link -> JIT -> reject.
5. Federation uniqueness is enforced at the DB level on `(realm_id, provider_id, subject)`.
6. PKCE is required by default and admin-configurable per provider.
7. Audit events for the IdP lifecycle land in the existing telemetry pipeline under a dedicated `category=idp` family.
8. `core.auth.oauth_idp` supports both static `provider_alias` config and runtime alias input from a preceding `collect_idp_choice` node; runtime input wins when present.
9. Per-realm + per-provider throttle on `/auth/oauth/{alias}/start` (default 30 starts per IP per provider per 10 minutes), realm-configurable on `RealmIdpSettings` using the recovery rate-limit pattern.
10. Email-claim resolution defaults to OIDC `email`+`email_verified`; per-preset `claim_fetch_strategy` (`id_token` default, `endpoint` for providers like GitHub) isolates provider quirks before claim mapping runs.
11. JIT email-verification trust: IdP `email_verified=true` is trusted only when the provider has `require_verified_email=true`; otherwise JIT users go through ReAuth's standard email-verification path. Email auto-link to existing users always requires verified email.
12. Federation row deletion is self-service for the end user, guarded by `idp_minimum_remaining_factor`; admins can also delete from the user detail view.
13. Brokering is a branch inside existing flow types — no new `auth_oauth_broker` flow type.
14. Discovery and JWKS caches live inline on `identity_providers` (`metadata_cache_json`+`metadata_cached_at`, `jwks_cache_json`+`jwks_cached_at`); no separate cache table in v1.
15. ReAuth-realm-as-upstream-IdP for another ReAuth deployment is supported through the generic `custom-oidc` preset; no bespoke cross-realm broker entity in v1.
