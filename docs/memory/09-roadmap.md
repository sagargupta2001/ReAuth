# Roadmap

This is the cross-cutting roadmap. Feature-specific roadmaps live in `reauth/docs/memory/roadmaps/`.

## Focus Shift: Core Platform Roadmap (Phased)

### Phase 1: OIDC Hardening & Flow Engine Maturity (Foundation)
- Roadmap: `reauth/docs/memory/roadmaps/oidc-flow-engine.md`
- Goal: bring protocol compliance and the engine to production-grade security.

### Phase 2: Must-Have Identity Flows (MVP Parity)
- Roadmap: `reauth/docs/memory/roadmaps/identity-flows.md`
- Goal: registration, email verification, credential recovery, and MFA.

### Phase 3: Developer Experience (SDKs)
- Roadmap: `reauth/docs/memory/roadmaps/developer-experience.md`
- Goal: React SDK and Node.js/Express SDK for 5-minute integration.

### Phase 4: Enterprise Identity Brokering (Post-MVP)
- Roadmap: `reauth/docs/memory/roadmaps/identity-brokering.md`
- Goal: inbound social login and LDAP/AD integrations.

### Phase 5: Advanced IAM Protocols
- Roadmap: `reauth/docs/memory/roadmaps/advanced-iam.md`
- Goal: token exchange and back-channel logout.

### Cross-Cutting Architecture
- Remove Plugin System: `reauth/docs/memory/roadmaps/remove-plugin.md`

## Risks and dependencies
- Flow engine pause/resume is a prerequisite for email verification and recovery flows.
- OIDC hardening requires token family tracking and stricter client type validation.
- Embedded scripting safety and sandboxing will gate advanced extensibility.
- SDKs depend on stable, well-documented API contracts and error semantics.
