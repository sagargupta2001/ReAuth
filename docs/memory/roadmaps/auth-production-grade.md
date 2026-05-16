# Feature Roadmap: Production-Grade Auth Flows

## Goal
- Turn ReAuth's auth stack into a production-grade system across browser login, registration, recovery, MFA, OIDC, theming, and extensibility.
- Make flow-canvas nodes map cleanly to Fluid pages wherever a node renders UI.

## Aligned roadmap inputs
- `docs/memory/roadmaps/identity-flows.md`
- `docs/memory/roadmaps/oidc-flow-engine.md`
- `docs/memory/roadmaps/oidc-future-enhancements.md`
- `docs/memory/roadmaps/theme-engine.md`
- `docs/memory/roadmaps/flow-extensibility.md`
- `docs/memory/roadmaps/flow-resume-design.md`

## Current state (code-aligned)
- OIDC Phase 1 hardening is complete: discovery, `/userinfo`, PKCE S256, refresh-token rotation with reuse detection, and async pause/resume are already implemented.
- The flow executor already propagates a node `template_key` into the UI challenge context, and theme resolution already supports page lookup plus client override fallback.
- Fluid already has system pages for `login`, `register`, `forgot_credentials`, `verify_email`, `mfa`, `consent`, `magic_link_sent`, and `error`.
- The builder can assign a Fluid page key per node through `data.config.template_key`, but today that is effectively wired around authenticator-style UI nodes.
- Registration and reset flow bindings exist at the realm level, but their runtime behavior is still placeholder-grade and does not yet implement true account creation or credential recovery.
- The palette advertises `otp`, `condition`, and `script` nodes, but the runtime currently only registers executable workers for `core.auth.password` and `core.auth.cookie`.
- The only fully wired runtime screen is the Fluid-backed login screen plus the waiting screen for async actions.
- OIDC client secrets are still stored directly, which is acceptable for local development but not a production-grade endpoint posture.

## Feasibility of node -> Fluid page mapping
- Feasible now for any node that already suspends to UI: the data model and executor path already support explicit `template_key` values.
- Not fully feasible yet for "every node" in the literal sense:
  - Logic nodes do not currently execute through dedicated workers in the main flow executor path.
  - Non-UI nodes do not need a page, so the system needs a node capability model instead of a blanket requirement.
  - The screen registry is still mostly password-login-centric, so new UI-capable node types need explicit screen contracts.
- Recommended direction:
  - Treat page binding as a capability of "UI-capable nodes", not all nodes.
  - Introduce explicit node metadata such as `ui_surface`, `default_template_key`, and `screen_id`.
  - Keep `template_key` in node config for backward compatibility, then migrate toward a typed `ui` block.

## Now
- Stabilize the current flow builder/runtime contract.
  - Keep node labels, runtime worker keys, and default template bindings aligned.
  - Stop exposing palette nodes as "available" unless they are executable, or implement their workers immediately.
  - Add template-gap validation for all UI-capable node types, not just password/OTP defaults.
- Ship real registration and forgot-credentials flows.
  - Registration: user creation, password policy, duplicate-account handling, and post-registration verification.
  - Registration policy: realm capabilities (registration enabled + default roles), master realm guard, and UI gating.
  - Recovery: request token, verify token, set new password, revoke existing sessions, and emit audit events.
  - Add public entry points and UX routing for register/recovery instead of relying on placeholder pages alone.
- Close production-grade OIDC gaps with the highest risk payoff.
  - Structured spec-aligned error responses.
  - Conformance coverage for authorize/token/userinfo/discovery.
  - Secret-handling improvements for confidential clients.
  - Audit events for OIDC failures, token reuse, and suspicious flows.

## Implementation checklist
- [x] OIDC Phase 1 hardening complete (PKCE S256, refresh rotation, discovery, `/userinfo`, async pause/resume).
- [x] Flow executor propagates `template_key` into UI context for Fluid page selection.
- [x] System Fluid pages exist for login/register/forgot/verify/mfa/consent/magic link/error.
- [x] Palette only exposes nodes that are executable in the runtime registry.
- [x] Node metadata includes `supports_ui` and `default_template_key` from the backend.
- [x] Flow builder consumes backend defaults for page binding and missing-template warnings.
- [x] Registration node implemented (`core.auth.register`) with UI binding to `register`.
- [x] Registration flow template uses the register node.
- [x] Public `/auth/register` and `/auth/register/execute` endpoints are live.
- [x] UI `/register` route wired to the shared auth executor.
- [x] Login page links to `/register` by default in system theme.
- [x] Session creation is realm-scoped (no hardcoded `master`).
- [x] Realm capability flags: `registration_enabled` + `default_registration_roles` stored per realm.
- [x] Policy guard: master realm cannot enable self-registration.
- [x] Flow/runtime exposes realm capabilities to UI context for Fluid gating (hide register link).
- [x] Registration worker assigns default roles on self-registration.
- [x] Reset flow entry routes and UI wiring (`/forgot-password` -> reset flow).
- [x] Forgot-credentials runtime node and flow (token issuance + validation + password reset).
- [x] Recovery delivery channel (SMTP email) wired to async resume.
- [x] Recovery request rate limits + audit events.
- [x] Session revocation on password reset.
- [x] Awaiting-action UI shows expiration + resend entry point.
- [x] Recovery resend + expiration UX copy.
- [x] Per-realm recovery settings (token TTL, rate limits, session revocation, templates).
- [x] Recovery identifier supports username or email (user email stored on domain model).
- [x] Recovery UX route and API path (`/forgot` or `/recover`) wired to flow executor.
- [x] Realm security headers (X-Frame-Options, CSP, etc.) configurable per realm and applied on auth/OIDC responses.
- [x] OTP/email verification node with async pause/resume and resend support.
- [x] Consent node for OIDC scopes.
- [x] Condition node execution support in runtime (not just palette metadata).
- [x] OIDC spec-aligned error payloads across authorize/token/userinfo.
- [ ] OIDC conformance harness coverage in CI.
- [x] Confidential-client secret storage/rotation plan implemented.
- [x] Action status endpoint for async flows (`GET /api/realms/{realm}/auth/action-status?token=...`).
- [x] Awaiting-action polling with backoff and auto-redirect on consume.
- [x] Awaiting-action UX copy for auto-redirect (“Recovery confirmed, redirecting…”).

## Master admin bootstrapping (production posture)
### Design summary
- First-run setup mode is required; no default credentials are ever enabled.
- A one-time setup token is printed to the console on first boot.
- `/setup` is enabled only until the first master admin is created, then permanently disabled.
- Admin recovery uses the normal Flow system + a CLI break-glass command.

### Implementation checklist
- [x] First-run detection: determine whether master realm has zero users at startup.
- [x] Console setup token: generate high-entropy token and print to stdout.
- [x] Setup route: add `/api/system/setup` guarded by the token.
- [x] Setup UI: prompt for token, then collect username/password for the first admin.
- [x] Master seed: create master realm + first admin with system-level permissions.
- [x] Seal system: disable setup routes after first admin creation.
- [x] Break-glass CLI: `reauth admin reset-password --user <username>`.

## Decisions (locked)
- Recovery flows are **single entry-point** with internal branching for username vs password recovery.
- OIDC consent is a **dedicated node** with OIDC-specific metadata handling.
- OIDC client secrets are **encrypted at rest** via a master key (AES-GCM), not hashed.

## Next
- [x] Generalize node/page binding into a first-class flow capability.
  - [x] Node metadata includes `ui_surface`, `default_template_key`, and allowed page categories.
  - [x] Inspector exposes Fluid-page binding for any UI-capable node.
  - [x] Binding stored in `config.ui.page_key` with `template_key` fallback.
  - [x] Missing-template warnings respect UI bindings + node defaults.
  - [x] Publish-time validation enforces page existence + category alignment for UI-capable nodes.
- Add more executable node types.
  - OTP / email verification node.
  - Recovery token node.
  - Consent node for OIDC scopes/claims approval.
  - Condition node with a real evaluator.
  - Webhook / external decision node built on async pause/resume.
- Introduce MFA baseline.
  - TOTP enrollment and verification.
  - Realm/client policy hooks for required MFA.
  - Backup/recovery path design before enforcement.

## Later
- Expand advanced OIDC/security capabilities.
  - Scope-aware `/userinfo` claims filtering.
  - Consent persistence and richer prompt handling.
  - JAR/PAR, optional DPoP or MTLS, and more complete client-auth options.
- Add stronger assurance flows.
  - WebAuthn.
  - Step-up auth.
  - Risk-based policies and adaptive challenges.

## Production track by workstream
- Flow engine
  - One executable contract per node type.
  - Typed UI metadata instead of ad hoc screen matching.
  - Publish-time validation that runtime workers, theme pages, and graph outputs all exist.
- Auth journeys
  - Browser login, registration, verification, recovery, MFA, and consent all use the same graph engine.
  - Async resume is the default building block for email and approval steps.
- OIDC
  - Conformance-tested authorization code flow.
  - Hardened secret handling, auditing, and clearer operator-facing diagnostics.
- Fluid integration
  - Every UI-capable auth node can resolve a page in the active theme.
  - Default system pages stay available as safe fallback.
## Extensibility follow-up
- [x] Add subflow call/return semantics for reusable flow composition.
- [ ] Add end-to-end `call_subflow` action coverage from Fluid Action Binder.

## Exit criteria for "production grade"
- Registration, recovery, login, verification, and MFA all work end-to-end with audit coverage.
- Every UI-capable node has an explicit screen contract and a Fluid page binding story.
- Palette nodes are either executable in runtime or hidden from the builder.
- OIDC endpoints emit spec-aligned errors and have strong automated integration coverage for the flows ReAuth chooses to support.
- Confidential-client secrets are handled with a production-safe storage/rotation story.
- Suspicious auth activity is observable through logs/events with actionable context.

## Risks / dependencies
- Email delivery, template rendering, and anti-abuse controls gate registration/recovery.
- The current executor architecture needs a small generalization to support non-authenticator workers cleanly.
- UI/page binding needs stable screen contracts; otherwise page selection becomes decorative instead of executable.

## Open questions
- Should setup tokens be time-bound or single-use only (or both)?
- Where should setup tokens be stored (memory-only vs DB with TTL)?
- Do we allow remote setup in environments where stdout is not accessible?
