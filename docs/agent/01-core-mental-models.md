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
- Scripts are for constrained customization, not for replacing protocol primitives, storage, or security boundaries.
- New features should compose with existing flows, themes, and realm scoping instead of bypassing them.

## Change discipline

- Prefer extending existing bounded areas before inventing new top-level modules.
- Keep contracts typed and explicit.
- Keep migrations forward-only.
- Keep docs and specs in sync with implementation.
