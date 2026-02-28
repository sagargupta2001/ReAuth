# Feature Roadmap: OIDC Hardening & Flow Engine Maturity

Status: Complete (Phase 1 scope)

## Goal
- Bring OIDC to strict compliance and make the flow engine secure and resumable.

## Current state (code-aligned)
- OIDC authorize/token/JWKS endpoints exist.
- OIDC discovery (`/.well-known/openid-configuration`) is implemented.
- `/userinfo` endpoint validates access tokens and returns user claims.
- PKCE S256 enforcement for public clients is implemented (configurable).
- Refresh tokens rotate on use with family invalidation on reuse.
- Flow engine supports challenge/success/failure plus async waiting states.
- UI step resumption already works via `auth_sessions` + login session cookies (refreshing keeps the current node).
- Async pause/resume (email verification, magic link, webhook) is implemented via action tokens and waiting UI.
- Engine-level brute force protection/lockout is implemented in the password authenticator.

## Now
- Enforce PKCE for public clients; reject missing code_challenge.
- Add `/.well-known/openid-configuration` with spec-complete metadata.
- Ensure `/authorize`, `/token`, `/jwks`, and `/userinfo` adhere to OIDC spec.
- Implement refresh token rotation with token family invalidation on reuse.
- Add engine-level rate limiting and lockout (example: 5 failed attempts = 15 minutes).
- Persist flow execution state for secure pause/resume (email verification, recovery).

## Phase 1 Implementation Checklist
- [x] Add `/.well-known/openid-configuration` endpoint with issuer, endpoints, and supported features.
- [x] Add `/userinfo` endpoint and validate access tokens.
- [x] Enforce PKCE for public clients at `/authorize`.
- [x] Reject `code_challenge_method` not equal to `S256`.
- [x] Require `code_verifier` when a `code_challenge` exists.
- [x] Add refresh token family model and rotation with reuse detection.
- [x] Invalidate entire token family on reuse detection.
- [x] Add brute-force protection: attempts counter + lockout window in the password authenticator.
- [x] Implement async pause/resume using action tokens (see `flow-resume-design.md`).
- [x] Store execution state and last UI output for suspend/resume flows (async).
- [x] Add config defaults for PKCE enforcement, lockout threshold, and lockout duration.
- [x] UI settings/toggles to override PKCE enforcement, lockout thresholds, and lockout duration.

## Future enhancements
- See `reauth/docs/memory/roadmaps/oidc-future-enhancements.md`.

## Risks / dependencies
Resolved/mitigated in Phase 1:
- Token family rotation is persisted and enforced (reuse invalidates the family).
- Public vs confidential client classification is enforced by client secret presence.
- Resume tokens are time-bound and replay-safe in the flow engine.

## Decisions (best-practice defaults)
- Public client detection: `client_secret == null` => public, otherwise confidential.
- Refresh token rotation: enabled for all clients; reuse invalidates the entire family.
- Lockout counters: stored in the primary DB for durability and consistent enforcement.
