# Core Mental Models

These are the non-negotiable product and engineering constraints for ReAuth.

## Product identity

- ReAuth is a single-binary identity provider.
- The backend is Rust-first and the UI can be embedded into the binary.
- The system is self-hosted, SQLite-first, and optimized for low operational overhead.
- ReAuth is inspired by Keycloak, but it is not trying to copy Keycloak feature-for-feature.

## Architectural stance

- Backend architecture is hexagonal:
  - domain holds core business concepts and rules
  - application orchestrates use cases
  - ports define boundaries
  - adapters implement infrastructure and delivery concerns
- Frontend architecture follows Feature-Sliced Design:
  - `app`, `pages`, `widgets`, `features`, `entities`, `shared`

## Operational stance

- Prefer standalone capabilities over introducing new infrastructure.
- Do not add dependencies on external operational systems such as Redis, Kafka, or worker fleets unless the product direction explicitly changes.
- Favor in-process, SQLite-backed, or file-backed solutions when they fit the requirement.
- If a feature appears to require new infrastructure, first ask whether the feature can be expressed inside the current single-binary mental model.

## Feature design stance

- Prefer explicit domain and flow primitives over clever custom code.
- Built-in security-sensitive auth capabilities should be first-class nodes or services, not scripts.
- New features should compose with existing flows, themes, and realm scoping instead of bypassing them.

## Feature availability model

Every auth capability should be designed across three layers:

- System/operator capability:
  - answers "can this deployment support this at all?"
  - examples: RP ID/origin config for passkeys, SMTP/public URL for magic links
- Realm policy:
  - answers "is this allowed in this realm, and with what security/product defaults?"
  - examples: feature enabled flags, TTLs, fallback rules, rate limits
- Flow composition:
  - answers "where in the journey do users actually experience this?"
  - examples: browser login includes a passkey branch, reauth requires passkey, recovery excludes magic link

Rules:

- Settings decide what is allowed.
- Flows decide what is experienced.
- Nodes implement the capability.
- Do not force admins to delete nodes just to temporarily disable a realm feature.
- Do not rely on a single global toggle when different flows need different experiences.
- Security and product policy belongs primarily at the realm layer.
- Journey placement and branching belongs at the flow layer.

## Admin UX stance

- ReAuth should support a simple mode and an advanced mode for major auth capabilities.
- Simple mode should expose feature toggles and recommended presets.
- Advanced mode should expose explicit flow composition in the builder.
- Builder UX should reflect feature availability:
  - unavailable nodes are hidden or clearly disabled when system/realm prerequisites are missing
  - publish-time validation should catch flows that depend on disabled capabilities

## Change discipline

- Prefer extending existing bounded areas before inventing new top-level modules.
- Keep contracts typed and explicit.
- Keep migrations forward-only.
- Keep docs and specs in sync with implementation.
