# Feature Roadmap: Advanced IAM Protocols

## Goal
- Elevate ReAuth from simple login to distributed systems security.

## Current state
- Standard OAuth/OIDC flows implemented without token exchange or back-channel logout.

## Now
- OAuth 2.0 Token Exchange (RFC 8693) for service-to-service token scoping.
- Scoped token issuance for downstream services and least-privilege propagation.

## Next
- Session management with back-channel logout.
- Server-to-server logout webhooks to connected clients.
- Admin session revocation that terminates downstream sessions.

## Later
- Front-channel logout and session management UI.
- Session introspection endpoint for compliance.

## Risks / dependencies
- Requires stable session model and token family tracking.
- Client integrations must support back-channel signals.

## Open questions
- Token exchange policy model and scope mapping rules.
- Back-channel delivery transport and retry semantics.
