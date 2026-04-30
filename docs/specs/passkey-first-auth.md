# Spec: Passkey-First Authentication

> Distilled from: product direction discussion / 2026-04-29
> Status: Draft

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
3. The system must support both discoverable-credential sign-in and identifier-assisted sign-in when a user has no discoverable credential or cancels the ceremony.
4. WebAuthn challenges must be short-lived, one-time, realm-scoped, and bound to the expected RP ID and origin set.
5. Successful assertion must authenticate the correct realm-scoped user and continue the active flow without requiring a password.
6. Passkey enrollment must only succeed for an authenticated or explicitly verified user session and must use `excludeCredentials` to prevent duplicate registration.
7. Reauthentication flows must be able to require passkey first and only fall back to password if realm policy explicitly allows fallback.
8. The system must persist credential metadata needed for future assertions, including credential ID, public key, sign counter, transports, backup eligibility/state, and timestamps.
9. If the browser or platform does not support WebAuthn, the flow must degrade cleanly to the next configured branch.
10. All passkey registration and assertion outcomes must emit audit events for success, failure, cancellation, and replay/mismatch attempts.
11. Passkey availability must follow ReAuth's capability scoping model:
   - system/operator capability determines whether passkeys can run at all
   - realm policy determines whether passkeys are allowed and what fallback rules apply
   - flow composition determines where passkeys appear in specific journeys

**Edge cases:**
- A challenge is replayed, expired, or submitted from the wrong origin.
- A user has no discoverable credential but is allowed to fall back to identifier + password.
- A previously enrolled credential is deleted or cloned and the counter/state becomes suspicious.
- A realm enables passkey-first login before any users have enrolled passkeys.

---

## Domain Changes

### New Entities (if any)

```text
WebAuthnCredential
  - id: uuid - primary identifier
  - realm_id: uuid - owning realm
  - user_id: uuid - owning user
  - credential_id: bytes/text - WebAuthn credential identifier
  - public_key_cose: bytes/text - stored public key material
  - sign_count: i64 - latest observed counter
  - transports_json: text - optional transports metadata
  - backed_up: bool - backup state from authenticator data
  - backup_eligible: bool - whether credential supports backup
  - aaguid: text? - authenticator AAGUID when available
  - friendly_name: text? - admin/user-visible label
  - created_at: timestamp - creation timestamp
  - updated_at: timestamp - last use/update timestamp

WebAuthnChallenge
  - id: uuid - primary identifier
  - realm_id: uuid - owning realm
  - auth_session_id: uuid - active auth session
  - user_id: uuid? - present for identifier-assisted flows
  - challenge_type: text - registration | assertion | reauth
  - challenge: text - encoded challenge payload
  - rp_id: text - relying party id used for ceremony
  - expires_at: timestamp - one-time validity window
  - consumed_at: timestamp? - replay protection marker
```

### Modified Entities (if any)

```text
Realm
  + passkeys_enabled: bool - enables passkey features for the realm
  + passkey_policy_json: text? - RP ID/origin/fallback/timeout policy

AuthenticationSession
  + webauthn_context: json? - active ceremony metadata for current step
```

### New Value Objects (if any)

```text
PasskeyPolicy - realm-level policy for RP ID, allowed origins, discoverable credential preference, and password fallback.
WebAuthnAssertionResult - normalized verification result returned by the WebAuthn node.
WebAuthnRegistrationResult - normalized registration result returned by the enrollment node.
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/...` | Add WebAuthn credential/challenge models and passkey policy value objects |
| `application/...` | Add WebAuthn verification service, challenge lifecycle, enrollment/authentication node logic, and audit integration |
| `adapters/web/...` | Add public endpoints for registration/assertion options and verification, plus browser flow handlers |
| `adapters/persistence/...` | Add repositories and SQLite implementations for passkey credentials/challenges |
| `ui/src/features/...` | Add passkey-first browser UX, enrollment prompts, cancellation/fallback states, and reauth handling |

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_create_webauthn_credentials.sql - store realm-scoped passkey credentials
YYYYMMDDHHMMSS_create_webauthn_challenges.sql - store one-time WebAuthn challenges
YYYYMMDDHHMMSS_add_realm_passkey_policy.sql - add passkey feature flags and policy columns
```

### Data Notes

- All passkey data must be realm-scoped and user-scoped.
- `credential_id` should be unique within a realm.
- Challenges must be indexed by auth session and expiry for fast cleanup and replay protection.
- Existing realms should default to `passkeys_enabled = false`.

---

## API Changes

### New HTTP Endpoints

```text
POST /api/realms/{realm}/auth/webauthn/assertion/options
  Request:  { auth_session_id?: uuid, identifier?: string, intent?: "login" | "reauth" }
  Response: { request_id: string, public_key: object, fallback_allowed: bool }
  Auth:     public

POST /api/realms/{realm}/auth/webauthn/assertion/verify
  Request:  { auth_session_id?: uuid, credential: object }
  Response: { result: "continue" | "challenge" | "failure", execution?: object }
  Auth:     public

POST /api/realms/{realm}/auth/webauthn/registration/options
  Request:  { auth_session_id?: uuid, user_label?: string }
  Response: { request_id: string, public_key: object }
  Auth:     auth required or verified auth session required

POST /api/realms/{realm}/auth/webauthn/registration/verify
  Request:  { request_id: string, credential: object, friendly_name?: string }
  Response: { result: "continue" | "failure", credential_id?: string }
  Auth:     auth required or verified auth session required
```

### Modified Endpoints (if any)

```text
GET /api/realms/{realm}/auth/login
  Changed response: may include passkey-first UI context and fallback policy

POST /api/realms/{realm}/auth/login/execute
  Added to request: signal payloads that can trigger passkey fallback branches
```

---

## Flow / Auth Impact

Use this section when the feature touches login, registration, recovery, OIDC, or flow builder behavior.

- Flow types affected: browser, registration
- New nodes: `core.auth.passkey_assert`, `core.auth.passkey_enroll`
- Existing nodes modified: `core.auth.password`, browser flow templates, registration flow templates
- Async pause/resume impact: none for the WebAuthn ceremony itself; normal flow session continuation still applies
- Theme or Fluid page impact: login and reauth pages need passkey-first CTAs, unsupported-browser states, and enrollment surfaces

---

## Availability / Admin UX

Use this section for capabilities that can be turned on/off or placed differently across journeys.

- System/operator prerequisites:
  - configured WebAuthn RP ID
  - configured allowed origins for the RP
  - HTTPS-capable deployment outside local development
- Realm policy:
  - `passkeys_enabled`
  - passkey policy for timeout, discoverable credential preference, and password fallback allowance
  - realm policy decides whether password fallback is allowed at all
- Flow composition:
  - browser flow may use passkey-first with password fallback
  - reauth flow may use passkey-required with no fallback branch
  - registration flow may include post-registration passkey enrollment
- Builder behavior:
  - hide or disable passkey nodes when system prerequisites are missing
  - warn or fail publish when a flow uses passkey nodes while the realm policy disables passkeys
- Simple mode UX:
  - realm settings page exposes "Enable passkeys"
  - optional preset such as "Use recommended passkey-first browser flow"
- Advanced mode UX:
  - flow builder exposes explicit passkey nodes and fallback branches
  - admins can allow passkeys at the realm level without placing them in every flow

---

## Implementation Phases

### Phase 1: Passkey Foundation

- Add domain models, repositories, and migrations for WebAuthn credentials and challenges.
- Add system-level config for RP ID and allowed origins.
- Add realm-level passkey settings and policy storage.
- Add WebAuthn service abstractions for challenge issuance and verification.

### Phase 2: Assertion Primitive

- Implement `core.auth.passkey_assert`.
- Add public assertion-options and assertion-verify endpoints.
- Support identifier-assisted and discoverable-credential login attempts.
- Emit audit events for assertion success/failure/cancellation/replay.

### Phase 3: Enrollment Primitive

- Implement `core.auth.passkey_enroll`.
- Add registration-options and registration-verify endpoints.
- Support enrollment for authenticated or otherwise verified users only.
- Persist credential metadata and block duplicate registrations with `excludeCredentials`.

### Phase 4: Passkey-First Browser UX

- Add simple-mode realm toggle and recommended browser-flow preset.
- Update browser login UI for passkey-first prompts, unsupported-browser states, and password fallback.
- Add builder validation and node availability behavior tied to system and realm capability checks.

### Phase 5: Reauth + Hardening

- Add passkey-first or passkey-required reauth flow support.
- Harden suspicious counter/backed-up state handling and operator diagnostics.
- Add cleanup jobs, additional metrics, and documentation for rollout/fallback operations.

### Immediate Goal

- Start with Phase 1 and Phase 2.
- Do not redesign the entire login UX before the assertion primitive works end to end.

---

## Test Scenarios

Scenarios that must pass before the feature is complete:

1. **Passkey-first login happy path**
   - Given: a realm with passkeys enabled and a user with an enrolled credential
   - When: the user starts the browser flow and completes a valid assertion
   - Then: the user is authenticated without entering a password

2. **Fallback to password**
   - Given: a realm with passkeys enabled and password fallback allowed
   - When: the user has no discoverable credential or cancels the ceremony
   - Then: the flow offers identifier/password fallback without breaking the session

3. **Registration and duplicate prevention**
   - Given: an authenticated user enrolling a passkey
   - When: the user attempts to register the same authenticator twice
   - Then: the second registration is rejected via `excludeCredentials`

4. **Challenge replay / mismatch**
   - Given: a previously issued or wrong-origin challenge
   - When: a client submits a stale or mismatched assertion
   - Then: verification fails, the challenge is not reusable, and an audit event is written

---

## Out of Scope

List nearby but intentionally excluded work:

- Attestation trust-chain policy and enterprise attestation review workflows
- Native mobile SDK ceremony support outside the browser
- Backup code design
- Risk-based passkey policy orchestration beyond basic fallback rules

---

## Open Questions

- [ ] Do we allow identifier-less sign-in only, or always keep identifier-assisted fallback in the first pass?
- [ ] Do we store WebAuthn challenges in a dedicated table or reuse existing auth action persistence with a new action type?
- [ ] What is the first-pass reauth policy window for sensitive actions?
