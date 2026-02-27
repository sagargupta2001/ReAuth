# Feature Roadmap: OIDC Hardening & Flow Engine Maturity

## Goal
- Bring OIDC to strict compliance and make the flow engine secure and resumable.

## Current state (code-aligned)
- OIDC authorize/token/JWKS endpoints exist.
- PKCE S256 is supported but not enforced for public clients.
- Refresh tokens exist without rotation on use.
- Flow engine supports challenge/success/failure.
- UI step resumption already works via `auth_sessions` + login session cookies (refreshing keeps the current node).
- Async pause/resume (email verification, magic link, webhook) is implemented via action tokens and waiting UI.
- No engine-level brute force protection or lockout policy.

## Now
- Enforce PKCE for public clients; reject missing code_challenge.
- Add `/.well-known/openid-configuration` with spec-complete metadata.
- Ensure `/authorize`, `/token`, `/jwks`, and `/userinfo` adhere to OIDC spec.
- Implement refresh token rotation with token family invalidation on reuse.
- Add engine-level rate limiting and lockout (example: 5 failed attempts = 15 minutes).
- Persist flow execution state for secure pause/resume (email verification, recovery).

## Phase 1 Implementation Checklist
- [ ] Add `/.well-known/openid-configuration` endpoint with issuer, endpoints, and supported features.
- [ ] Add `/userinfo` endpoint and validate access tokens.
- [ ] Enforce PKCE for public clients at `/authorize`.
- [ ] Reject `code_challenge_method` not equal to `S256`.
- [ ] Require `code_verifier` when a `code_challenge` exists.
- [x] Add refresh token family model and rotation with reuse detection.
- [x] Invalidate entire token family on reuse detection.
- [x] Add brute-force protection: attempts counter + lockout window in the password authenticator.
- [x] Implement async pause/resume using action tokens (see `flow-resume-design.md`).
- [x] Store execution state and last UI output for suspend/resume flows (async).
- [x] Add config defaults for PKCE enforcement, lockout threshold, and lockout duration.
- [x] UI settings/toggles to override PKCE enforcement, lockout thresholds, and lockout duration.

## Next
- Add OIDC compliance tests (conformance harness).
- Add structured error responses aligned with the OIDC spec.
- Add clock skew handling and auditing for token reuse detection.

## Later
- Optional DPoP or MTLS support for high-security deployments.

## Risks / dependencies
- Token family rotation requires schema updates and migrations.
- Public vs confidential client classification must be explicit and enforced.
- Resume tokens must be signed, time-bound, and replay-safe.

## Open questions
- Public client detection rules and overrides.
- Rotation policy for confidential clients.
- Storage for lockout counters (DB vs cache).
