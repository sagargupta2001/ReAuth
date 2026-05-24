# Feature Roadmap: Enterprise Identity Brokering

## Goal
- Broker external identities into ReAuth for social login and enterprise directory auth.

## Current state
- Inbound OAuth2 / OIDC identity brokering is implemented for realm-scoped providers.
- Current broker surface covers provider CRUD, runtime login/link/JIT flows, provider buttons, admin activity, unlink protection, cache refresh, and rollout diagnostics.

## Now
- Stabilize and operate the inbound brokered-login surface.
- Canonical rollout guidance lives in `docs/memory/21-identity-brokering-operations.md`.

## Next
- LDAP and Active Directory integration with group-to-role mapping.
- On-the-fly authentication with directory bind and caching.

## Later
- SCIM provisioning and lifecycle hooks.
- Multi-IdP routing by realm or client policy.

## Risks / dependencies
- Requires stable user identity linking model.
- LDAP schema variance and mapping complexity.
- Security: token validation, replay protection, and audit trails.

## Open questions
- Default account linking behavior vs explicit consent.
- How to map external groups to ReAuth roles at scale.
