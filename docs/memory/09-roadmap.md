# Roadmap

This is the cross-cutting roadmap. Feature-specific roadmaps live in `reauth/docs/memory/roadmaps/`.

## Focus Shift: Core Platform Roadmap (Phased)

### Phase 0: Plugin Removal (Completed)
- Status: Complete
- Roadmap: `reauth/docs/memory/roadmaps/remove-plugin.md`
- Goal: fully remove plugin system and simplify to a single-binary core.

### Phase 1: OIDC Hardening & Flow Engine Maturity (Foundation)
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/oidc-flow-engine.md`
- Goal: bring protocol compliance and the engine to production-grade security.

### Phase 2: Native Extensibility (Theme + Scripting)
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/theme-engine.md`
- Roadmap: `reauth/docs/memory/roadmaps/embedded-scripting.md`
- Goal: deliver a theme engine + embedded scripting runtime that replaces plugin extensibility.

### Phase 3: Must-Have Identity Flows (MVP Parity)
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/identity-flows.md`
- Goal: registration, email verification, credential recovery, and MFA.

### Phase 4: Developer Experience (SDKs)
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/developer-experience.md`
- Goal: React SDK and Node.js/Express SDK for 5-minute integration.

### Phase 5: Enterprise Identity Brokering (Post-MVP)
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/identity-brokering.md`
- Goal: inbound social login and LDAP/AD integrations.

### Phase 6: Advanced IAM Protocols
- Status: Planned
- Roadmap: `reauth/docs/memory/roadmaps/advanced-iam.md`
- Goal: token exchange and back-channel logout.

## Risks and dependencies
- Flow engine pause/resume is a prerequisite for email verification and recovery flows.
- OIDC hardening requires token family tracking and stricter client type validation.
- Theme engine depends on stable page/block schemas and resolver contracts.
- Embedded scripting safety and sandboxing will gate advanced extensibility.
- SDKs depend on stable, well-documented API contracts and error semantics.
