# Spec: Passkey-First Authentication

> Distilled from: product direction discussion / 2026-04-29
> Status: Implemented (Phase 1-5)

---

## User Story

As a realm admin, I want ReAuth to support passkey-first browser authentication so that end users can sign in and reauthenticate with native WebAuthn before falling back to passwords.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Enables passkeys, configures fallback policy, and adds passkey nodes to flows |
| End User | Signs in, enrolls a passkey, and reauthenticates with a passkey-first experience |
| OIDC Client | Receives successful browser login or step-up auth results from passkey-backed flows |
| Operator | Configures RP ID, origin policy, and rollout defaults |

---

## Business Rules

1. Passkeys must be delivered as built-in auth nodes and APIs, not as theme scripting or custom code.
2. When a realm enables passkey-first login, the browser flow must attempt WebAuthn assertion before password fallback.
3. The first release must support both discoverable-credential sign-in and identifier-assisted sign-in fallback.
4. Every passkey challenge must be short-lived, one-time, realm-scoped, session-scoped, and bound to RP ID + allowed origin policy.
5. Assertion success must authenticate the correct realm-scoped user and continue the active flow without requiring password input.
6. Passkey enrollment must only succeed for an authenticated or explicitly verified user session and must use `excludeCredentials` to prevent duplicate registration.
7. Reauth flows must be able to require passkey first and only fall back to password if realm policy explicitly allows fallback.
8. Reauth assurance must be time-bound: passkey reauth is considered fresh for a configurable window (default 300 seconds).
9. The system must persist credential metadata needed for future assertions, including credential ID, public key, sign counter, transports, backup eligibility/state, and timestamps.
10. If browser/platform WebAuthn is unavailable, the flow must degrade cleanly to the next configured branch without breaking session continuity.
11. Registration/assertion outcomes must emit audit events for success, failure, cancellation, replay, mismatch, and suspicious counter behavior.
12. Passkey availability must follow ReAuth capability scoping:
   - system/operator capability determines whether passkeys can run at all
   - realm policy determines whether passkeys are allowed and fallback behavior
   - flow composition determines where passkeys appear in journeys

**Edge cases:**
- Challenge replay, expiration, or wrong-origin submission.
- User has no discoverable credential and uses identifier-assisted fallback.
- Credential deletion or suspected clone (counter/state anomalies).
- Realm enables passkey-first before users have enrolled any passkeys.

---

## Domain Changes

### New Entities

```text
PasskeyCredential
  - id: uuid - primary identifier
  - realm_id: uuid - owning realm
  - user_id: uuid - owning user
  - credential_id_b64url: text - WebAuthn credential identifier
  - public_key_cose_b64url: text - COSE public key material
  - sign_count: i64 - latest observed counter
  - transports_json: text - optional transports metadata
  - backed_up: bool - backup state from authenticator data
  - backup_eligible: bool - whether credential supports backup
  - aaguid: text? - authenticator AAGUID when available
  - friendly_name: text? - admin/user-visible label
  - last_used_at: timestamp? - latest successful assertion time
  - created_at: timestamp
  - updated_at: timestamp

PasskeyChallenge
  - id: uuid - primary identifier returned to client as challenge_id
  - realm_id: uuid - owning realm
  - auth_session_id: uuid - active auth session
  - user_id: uuid? - present for identifier-assisted ceremonies
  - challenge_kind: text - authentication | enrollment | reauthentication
  - challenge_hash: text - SHA-256 hash of issued challenge
  - rp_id: text - relying party id used for ceremony
  - allowed_origins_json: text - origin snapshot at issuance time
  - expires_at: timestamp - one-time validity window
  - consumed_at: timestamp? - replay protection marker
  - created_at: timestamp

RealmPasskeySettings
  - realm_id: uuid - owning realm
  - enabled: bool - realm-level feature flag
  - allow_password_fallback: bool - fallback policy toggle
  - discoverable_preferred: bool - prefer discoverable UX when available
  - challenge_ttl_secs: i64 - default challenge lifetime
  - reauth_max_age_secs: i64 - reauth freshness window
  - updated_at: timestamp
```

### Modified Entities

```text
AuthenticationSession
  + passkey_context: json? - transient ceremony metadata for current step

RealmCapabilities
  + passkeys_enabled: bool
  + passkey_allow_password_fallback: bool
```

### New Value Objects

```text
PasskeyAssertionResult - normalized verification result for passkey authentication
PasskeyEnrollmentResult - normalized verification result for passkey registration
PasskeyPolicy - resolved policy from system config + realm settings
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/...` | Add passkey credential/challenge/settings models and policy value objects |
| `application/...` | Add passkey service for challenge issuance/verification, node orchestration, and audit integration |
| `adapters/web/...` | Add passkey auth endpoints and realm passkey settings endpoints |
| `adapters/persistence/...` | Add SQLite repositories for credentials/challenges/settings |
| `ui/src/features/...` | Add passkey-first login UX, enrollment UX, and fallback states |

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_create_passkey_credentials.sql - store realm-scoped passkey credentials
YYYYMMDDHHMMSS_create_passkey_challenges.sql - store one-time passkey challenges
YYYYMMDDHHMMSS_create_realm_passkey_settings.sql - store realm-level passkey policy
```

### Data Notes

- All passkey data is realm-scoped; credentials are also user-scoped.
- `passkey_credentials(credential_id_b64url, realm_id)` must be unique.
- `passkey_credentials(user_id, realm_id)` index is required for fast excludeCredentials generation.
- `passkey_challenges(realm_id, auth_session_id, challenge_kind)` and `passkey_challenges(realm_id, expires_at)` indexes are required.
- Existing realms default to passkeys disabled via `realm_passkey_settings.enabled = false`.
- Persist only `challenge_hash` (not raw challenge) to reduce replay value if DB data is leaked.

---

## Concurrency / Scalability Requirements

- Challenge consume must be atomic and single-winner:
  - `UPDATE ... SET consumed_at = now WHERE id = ? AND consumed_at IS NULL AND expires_at > now`.
  - Verification continues only when `rows_affected == 1`.
- Sign counter updates must be monotonic per credential:
  - Reject or flag suspicious assertions when observed counter regresses unexpectedly.
  - Use guarded update (`WHERE sign_count <= ?`) to avoid race regressions under concurrent assertions.
- Assertion verification must be cryptographic:
  - Verify authenticator signature over `authenticatorData || SHA-256(clientDataJSON)`.
  - Verify against the stored per-credential public key (`SubjectPublicKeyInfo` DER, base64url-encoded in persistence).
  - Baseline supported algorithms: ES256 and RS256.
- Credential registration must enforce uniqueness at DB level (not app-only checks).
- Cleanup runs in bounded batches to avoid long write locks on SQLite.
- All write paths must be transactionally grouped when mutating challenge + session + audit state.

---

## API Changes

### New HTTP Endpoints

```text
POST /api/realms/{realm}/auth/passkeys/authenticate/options
  Request:  { auth_session_id?: uuid, identifier?: string, intent?: "login" | "reauth" }
  Response: { challenge_id: string, public_key: object, fallback_allowed: bool }
  Auth:     public

POST /api/realms/{realm}/auth/passkeys/authenticate/verify
  Request:  { challenge_id: string, credential: object }
  Response: { result: "continue" | "challenge" | "failure", execution?: object }
  Auth:     public

POST /api/realms/{realm}/auth/passkeys/enroll/options
  Request:  { auth_session_id?: uuid, user_label?: string }
  Response: { challenge_id: string, public_key: object }
  Auth:     auth required or verified auth session required

POST /api/realms/{realm}/auth/passkeys/enroll/verify
  Request:  { challenge_id: string, credential: object, friendly_name?: string }
  Response: { result: "continue" | "failure", credential_id?: string }
  Auth:     auth required or verified auth session required

GET /api/realms/{id}/passkey-settings
  Response: realm passkey settings
  Auth:     protected, realm:read

PUT /api/realms/{id}/passkey-settings
  Request:  partial update for realm passkey settings
  Response: updated realm passkey settings
  Auth:     protected, realm:write

POST /api/realms/{id}/passkey-settings/recommended-browser-flow
  Request:  { enable_passkeys?: boolean } (default true)
  Response: { settings, browser_flow_version_id, browser_flow_version_number }
  Auth:     protected, realm:write

POST /api/realms/{id}/passkey-settings/recommended-registration-flow
  Request:  {}
  Response: { settings, registration_flow_version_id, registration_flow_version_number }
  Auth:     protected, realm:write

GET /api/realms/{id}/passkey-settings/analytics?window_hours=24&recent_limit=10
  Response: credential totals, challenge health, assertion/enrollment outcome counters, and recent failure events
  Auth:     protected, realm:read

GET /api/realms/{realm}/users/{id}/credentials
  Response: user credential inventory (password configured + flags + passkey list)
  Auth:     protected, user:write

PUT /api/realms/{realm}/users/{id}/credentials/password
  Request:  { password: string }
  Response: { status: "updated" }
  Auth:     protected, user:write

PUT /api/realms/{realm}/users/{id}/credentials/password-policy
  Request:  { force_reset_on_next_login?: boolean, password_login_disabled?: boolean }
  Response: { status: "updated" }
  Auth:     protected, user:write
  Policy:   password_login_disabled=true requires realm passkeys enabled and >=1 enrolled passkey for the user

PUT /api/realms/{realm}/users/{id}/credentials/passkeys/{credential_id}
  Request:  { friendly_name?: string | null }
  Response: { status: "updated" }
  Auth:     protected, user:write

DELETE /api/realms/{realm}/users/{id}/credentials/passkeys/{credential_id}
  Response: { status: "revoked" }
  Auth:     protected, user:write
```

### Modified Endpoints

```text
GET /api/realms/{realm}/auth/login
  Changed response: may include passkey-first UI context and fallback policy

POST /api/realms/{realm}/auth/login/execute
  Added to request/context: signal payloads that can trigger passkey fallback branches
```

---

## Flow / Auth Impact

- Flow types affected: browser, registration, step-up reauth
- New nodes: `core.auth.passkey_assert`, `core.auth.passkey_enroll`
- Existing nodes modified: `core.auth.password`, browser flow template, reauth flow template
- Async pause/resume impact: none for ceremony; normal flow session continuation applies
- Theme/Fluid impact: login and reauth pages need passkey CTAs, unsupported-browser state, and enrollment prompts
- Dedicated system pages: `passkey_assert`, `passkey_enroll` (auth category)

---

## Availability / Admin UX

- System/operator prerequisites:
  - configured WebAuthn RP ID
  - configured allowed origins
  - HTTPS-capable deployment outside local development
- Realm policy:
  - managed in `realm_passkey_settings`
  - includes enablement, fallback policy, challenge TTL, discoverable preference, reauth max age
- Flow composition:
  - browser flow can be passkey-first with password fallback
  - reauth flow can be passkey-required with no fallback
  - registration flow can include post-registration passkey enrollment
- Builder behavior:
  - hide/disable passkey nodes when system prerequisites are missing
  - block publish when a flow depends on passkey nodes but realm policy disables passkeys
- Simple mode UX:
  - realm settings exposes `Enable passkeys`
  - optional preset `Use recommended passkey-first browser flow`
- Advanced mode UX:
  - builder exposes explicit passkey nodes and fallback branches
  - realm can allow passkeys without forcing use in every flow

---

## Implementation Phases

### Phase 1: Foundation

- Add passkey credential/challenge/settings domain models, repositories, and migrations.
- Add system-level config for RP ID and allowed origins.
- Add realm passkey settings service + admin endpoints.
- Add passkey service abstractions for challenge issue/verify.

### Phase 2: Assertion Primitive

- Implement `core.auth.passkey_assert`.
- Add authenticate options/verify endpoints.
- Support discoverable and identifier-assisted attempts.
- Emit audit events for assertion success/failure/cancel/replay/suspicious counter.

### Phase 3: Enrollment Primitive

- Implement `core.auth.passkey_enroll`.
- Add enroll options/verify endpoints.
- Restrict enrollment to authenticated or verified sessions.
- Persist credential metadata and block duplicates with `excludeCredentials`.

### Phase 4: Passkey-First Browser UX

- Add simple-mode realm toggle + recommended browser-flow preset.
- Update login UX for passkey-first prompts, unsupported-browser states, and fallback.
- Add builder validation and node availability checks tied to capability layers.

### Phase 5: Reauth + Hardening

- Add passkey-required reauth support.
- Harden suspicious credential-state handling and operator diagnostics.
- Add challenge cleanup loop, metrics, and rollout docs.

Phase 5 closure criteria:
- Reauth freshness enforced by `reauth_max_age_secs` at runtime for reauth intent.
- Suspicious state events (invalid signature, challenge mismatch, counter regression) are audited.
- Background cleanup loop removes expired/consumed passkey challenges in bounded batches.
- Realm operators can inspect passkey diagnostics in UI observability via analytics endpoint.
- Rollout runbook (below) defines enablement, validation, and rollback steps.

### Immediate Goal

- Implement through Phase 2 first.
- Do not redesign entire login UX before assertion works end to end.

---

## Test Scenarios

1. **Passkey-first login happy path**
   - Given: realm passkeys enabled and user has enrolled credential
   - When: user starts browser flow and completes valid assertion
   - Then: user is authenticated without password entry

2. **Fallback to password**
   - Given: realm passkeys enabled and fallback allowed
   - When: user has no discoverable credential or cancels ceremony
   - Then: flow offers identifier/password fallback without session breakage

3. **Registration and duplicate prevention**
   - Given: authenticated user enrolling passkey
   - When: user attempts to register same authenticator twice
   - Then: second registration is rejected by `excludeCredentials` + DB uniqueness

4. **Challenge replay / mismatch**
   - Given: stale, consumed, or wrong-origin challenge
   - When: client submits assertion for verification
   - Then: verification fails, challenge cannot be reused, audit event is written

5. **Concurrent verify race**
   - Given: same challenge submitted concurrently from two requests
   - When: both verify calls hit backend near-simultaneously
   - Then: exactly one succeeds challenge consumption, the other fails safely

---

## Out of Scope

- Attestation trust-chain policy and enterprise attestation review workflow
- Native mobile SDK ceremonies outside browser WebAuthn
- Backup code design
- Risk-based adaptive policy orchestration beyond baseline fallback rules

---

## Rollout Runbook

1. Enable passkeys for pilot realm in `Realm Settings -> General -> Passkeys`.
2. Apply recommended browser flow preset to ensure passkey-first + password fallback.
3. Validate live traffic in `Realm Settings -> Observability -> Passkeys`:
   - assertion success/failure counters
   - pending/expired challenge counts
   - recent failure events
4. Keep fallback enabled during pilot; disable fallback only after assertion-success baseline stabilizes.
5. For incident rollback:
   - disable passkeys in realm settings, or
   - revert browser flow to previous published version.

---

## Decisions (Locked)

1. First release supports both identifier-less and identifier-assisted passkey login.
2. WebAuthn ceremonies use a dedicated `passkey_challenges` table (do not reuse `auth_session_actions`).
3. Reauth freshness window defaults to `300` seconds and is realm-configurable via passkey settings.
