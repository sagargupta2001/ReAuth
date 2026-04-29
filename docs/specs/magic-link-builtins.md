# Spec: Magic Link Built-Ins

> Distilled from: product direction discussion / 2026-04-29
> Status: Draft

---

## User Story

As a realm admin, I want ReAuth to provide built-in magic-link login nodes so that users can authenticate passwordlessly through the standard flow engine without custom scripting.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Enables magic-link login, configures TTL/rate limits, and places built-in nodes in flows |
| End User | Requests a login link, clicks the email link, and resumes authentication |
| OIDC Client | Receives the final login result after passwordless browser authentication |
| Operator | Configures SMTP/public URL and monitors audit/rate-limit events |

---

## Business Rules

1. Magic-link login must be implemented as built-in nodes on top of the existing async action token and resume infrastructure.
2. The browser flow must collect or confirm an identifier before a magic link is issued.
3. Every magic link token must be one-time, short-lived, realm-scoped, and bound to a specific auth session and action type.
4. Clicking a valid link must resume the correct flow session and continue the configured branch without requiring a password.
5. Expired, consumed, invalid, or cross-realm tokens must fail safely and never authenticate a user.
6. The system must support resend behavior using the existing action-status/resend pattern with realm-configurable cooldowns and rate limits.
7. Magic-link issuance must emit audit events for request, delivery attempt, resume success, resume failure, and resend.
8. Delivery must use the configured email delivery channel and produce a usable waiting-screen state even when the operator is using local-development fallback delivery.
9. Magic-link login must be available as a built-in browser-flow branch, not as a separate scripting capability.
10. Password or passkey fallback remains flow-configurable; magic link is an additional built-in primitive, not the only browser login mode.

**Edge cases:**
- A user requests multiple links and clicks an older one after a newer token has been sent.
- A token is consumed on one device while another waiting screen is still polling.
- A realm has SMTP misconfigured and delivery fails after issuance begins.
- A magic link is requested for an unknown or disabled account and the UI must avoid account enumeration.

---

## Domain Changes

### New Entities (if any)

```text
MagicLinkSettings
  - realm_id: uuid - owning realm
  - enabled: bool - feature flag
  - token_ttl_secs: i64 - link lifetime
  - resend_cooldown_secs: i64 - minimum resend gap
  - max_requests_per_window: i64 - anti-abuse limit
  - window_secs: i64 - rate-limit window
  - email_template_id: text? - optional template override
```

### Modified Entities (if any)

```text
AuthSessionAction
  ~ action_type: text - add dedicated magic-link action types
  ~ payload_json: text - include identifier, resume_path, and delivery metadata

Realm
  + magic_link_enabled: bool - realm toggle for passwordless login
  + magic_link_settings_json: text? - realm-level delivery and TTL settings
```

### New Value Objects (if any)

```text
MagicLinkRequest - normalized request to issue a magic-link action
MagicLinkDeliveryResult - delivery outcome and operator-visible diagnostics
MagicLinkPolicy - realm-level settings for TTL, resend, and rate limits
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/...` | Add magic-link settings/policy value objects and new action semantics |
| `application/...` | Add magic-link issue/resume/resend services and built-in node logic |
| `adapters/web/...` | Reuse current auth resume endpoints and add any needed public start/issue handlers |
| `adapters/persistence/...` | Persist realm magic-link settings and any additional action metadata |
| `ui/src/features/...` | Add identifier-first magic-link request UI, waiting state, resend UX, and consumed/expired states |

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_add_realm_magic_link_settings.sql - add realm-level magic-link flags and policy columns
YYYYMMDDHHMMSS_extend_auth_session_actions_for_magic_links.sql - add action typing/indexes needed for login links
```

### Data Notes

- Reuse `auth_session_actions` where possible instead of adding a parallel token table.
- Action rows for magic links must be indexed by token hash, expiry, and auth session.
- Realm settings should default to disabled.

---

## API Changes

### New HTTP Endpoints

```text
POST /api/realms/{realm}/auth/magic-link/request
  Request:  { auth_session_id?: uuid, identifier: string, intent?: "login" | "reauth" }
  Response: { result: "awaiting_action", execution: object }
  Auth:     public
```

### Modified Endpoints (if any)

```text
POST /api/realms/{realm}/auth/resume
  Changed behavior: accepts magic-link action tokens and resumes the associated login branch

POST /api/realms/{realm}/auth/resend
  Changed behavior: can resend pending magic-link login actions subject to cooldown and rate limits

GET /api/realms/{realm}/auth/action-status
  Changed behavior: reports consumed / expired state for magic-link actions
```

---

## Flow / Auth Impact

Use this section when the feature touches login, registration, recovery, OIDC, or flow builder behavior.

- Flow types affected: browser
- New nodes: `core.auth.collect_identifier`, `core.logic.issue_magic_link`, `core.logic.consume_magic_link` or equivalent built-in resume node contract
- Existing nodes modified: browser flow defaults, awaiting-action UI, email delivery service
- Async pause/resume impact: reuses existing async action token, resend, and polling flow
- Theme or Fluid page impact: login pages need “email me a link” actions; waiting pages use `magic_link_sent` or equivalent template

---

## Test Scenarios

Scenarios that must pass before the feature is complete:

1. **Magic-link login happy path**
   - Given: a realm with magic-link login enabled and a valid user identifier
   - When: the user requests a link and clicks the email link before expiry
   - Then: the flow resumes and authenticates the user without a password

2. **Resend and cooldown**
   - Given: a pending magic-link action
   - When: the user requests a resend before and after the cooldown window
   - Then: early resend is rejected and later resend succeeds

3. **Replay / expired token**
   - Given: an expired or already consumed magic-link token
   - When: the token is posted to the resume endpoint
   - Then: authentication does not occur and the UI shows a safe retry path

4. **Unknown account anti-enumeration**
   - Given: a login request for an unknown or disabled identifier
   - When: the user submits the identifier
   - Then: the UI response remains generic while audit/rate-limit behavior still applies internally

---

## Out of Scope

List nearby but intentionally excluded work:

- SMS-based magic links or OTP delivery
- Native mobile deep-link handling beyond browser resume URLs
- Rich multi-recipient approval flows
- Full passwordless account recovery redesign

---

## Open Questions

- [ ] Should login and reauth use the same magic-link action type or distinct built-in node types?
- [ ] Do we invalidate older pending magic links immediately when a new one is issued?
- [ ] Should magic-link login require a previously verified email, or can it act as implicit email verification on first login?
- [ ] Do we want a dedicated `/magic-link` public entry path later, or keep this entirely under the browser flow and existing `/resume` endpoint?
