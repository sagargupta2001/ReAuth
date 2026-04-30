# ADR: Capability Scoping Model For Auth Features

Status: Accepted
Date: 2026-04-29

## Context
ReAuth is building more built-in auth methods such as passkeys and magic links. These features create recurring product and admin UX questions:

- should a feature be enabled only by adding/removing flow nodes?
- should it be controlled only by a global toggle?
- how should multiple flows in the same realm behave when they need different user journeys?

Without an explicit rule, feature work will drift into inconsistent admin experiences and unclear ownership between system config, realm settings, and flow graphs.

## Decision
Adopt a three-layer capability scoping model for auth features:

- System/operator capability:
  - deployment-level prerequisites and configuration
  - examples: SMTP/public URL, WebAuthn RP ID/origins
- Realm policy:
  - realm-scoped enablement and security/product defaults
  - examples: enabled flags, TTLs, fallback policy, rate limits
- Flow composition:
  - where and how the capability appears in user journeys
  - examples: browser login branch, reauth requirement, enrollment after registration

Interpretation rules:

- Settings decide what is allowed.
- Flows decide what is experienced.
- Nodes implement the capability.

Admin UX rules:

- ReAuth should provide both a simple mode and an advanced mode where appropriate.
- Simple mode exposes feature toggles and recommended presets.
- Advanced mode exposes explicit flow composition in the builder.
- A realm-level disable must not require deleting nodes from flows just to turn the capability off temporarily.
- Builder validation should surface when a flow depends on a capability that is unavailable because of missing system prerequisites or realm policy.

## Alternatives considered
- Flow-only model:
  - rejected because operational or security policy should not require editing flow graphs for simple enable/disable changes.
- Toggle-only model:
  - rejected because different flows in the same realm may need different user journeys.
- System-global toggles only:
  - rejected because security and product behavior is primarily realm-scoped in ReAuth.

## Consequences
- New auth methods should usually ship with:
  - built-in nodes
  - realm-level policy/config
  - system-level prerequisite/config when needed
- Specs for auth features should explicitly document system/operator prerequisites, realm policy, and flow composition.
- Builder UX must understand capability availability and communicate disabled/unavailable nodes clearly.
- Publish-time validation should check both graph correctness and capability availability.
