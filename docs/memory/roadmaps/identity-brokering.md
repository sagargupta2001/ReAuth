# Feature Roadmap: Enterprise Identity Brokering

## Goal
- Broker external identities into ReAuth for social login and enterprise directory auth.

## Current state
- Local database authentication only.

## Now
- Inbound OIDC/OAuth2 social login (Google, GitHub, Microsoft).
- Account linking between external identities and local users.
- Profile mapping and conflict resolution strategy.

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
