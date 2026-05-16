# Roadmap

This is the cross-cutting roadmap. Feature-specific roadmaps live in `docs/memory/roadmaps/`.

## Focus Shift: Core Platform Roadmap (Phased)

### Current Primary Track: Production-Grade Auth Flows
- Status: In Progress
- Roadmap: `docs/memory/roadmaps/auth-production-grade.md`
- Unifies: identity flows, OIDC future hardening, Fluid page binding for auth nodes, and reusable built-in flow primitives.
- Goal: ship production-grade login, registration, recovery, MFA, OIDC, and extensible UI-capable auth nodes.
- Recent progress: recovery now has SMTP delivery, rate limiting, audit events, session revocation on reset, subflow composition, and Action Binder `call_subflow` execution.
- Next implementation focus: passkey-first auth primitives, magic-link flows, and node-contract migration strategy.

### Phase 0: Plugin Removal (Completed)
- Status: Complete
- Roadmap: `docs/memory/roadmaps/remove-plugin.md`
- Goal: fully remove plugin system and simplify to a single-binary core.

### Phase 1: OIDC Hardening & Flow Engine Maturity (Foundation)
- Status: Complete
- Roadmap: `docs/memory/roadmaps/oidc-flow-engine.md`
- Goal: bring protocol compliance and the engine to production-grade security.

### Phase 2: Native Extensibility (Fluid + Built-In Flow Primitives)
- Status: In Progress
- Roadmap: `docs/memory/roadmaps/theme-engine.md`
- Roadmap: `docs/memory/roadmaps/flow-extensibility.md`
- Roadmap: `docs/memory/roadmaps/flow-action-binding.md`
- Goal: deliver a theme engine plus reusable built-in auth/flow nodes that replace plugin extensibility.

### Phase 3: Must-Have Identity Flows (MVP Parity)
- Status: Planned
- Roadmap: `docs/memory/roadmaps/identity-flows.md`
- Goal: registration, email verification, credential recovery, and MFA.

### Phase 4: Developer Experience (SDKs)
- Status: Planned
- Roadmap: `docs/memory/roadmaps/developer-experience.md`
- Goal: React SDK and Node.js/Express SDK for 5-minute integration.

### Phase 5: Enterprise Identity Brokering (Post-MVP)
- Status: Planned
- Roadmap: `docs/memory/roadmaps/identity-brokering.md`
- Goal: inbound social login and LDAP/AD integrations.

### Phase 6: Advanced IAM Protocols
- Status: Planned
- Roadmap: `docs/memory/roadmaps/advanced-iam.md`
- Goal: token exchange and back-channel logout.

## Risks and dependencies
- Flow engine pause/resume is a prerequisite for email verification and recovery flows.
- Theme engine depends on stable page/block schemas and resolver contracts.
- SDKs depend on stable, well-documented API contracts and error semantics.
