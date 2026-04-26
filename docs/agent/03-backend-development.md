# Backend Development Guide

Use this guide for Rust/backend work.

## Architectural rules

- Keep business concepts in `src/domain`.
- Keep orchestration in `src/application`.
- Keep trait boundaries in `src/ports`.
- Keep infrastructure and delivery details in `src/adapters`.
- Keep startup composition in `src/bootstrap`.

## Implementation rules

- Prefer extending existing services and repositories before creating parallel ones.
- Prefer typed structs and enums over loosely shaped JSON except where the product is intentionally JSON-driven, such as flow graph payloads and theme payloads.
- Keep realm scoping explicit. Never hardcode `master` for runtime behavior.
- Prefer additive interfaces and config over ad hoc special cases.
- Security-sensitive auth/protocol features should be explicit runtime primitives, not hidden inside generic scripting.

## When modifying an existing feature

1. Find the current vertical slice:
   - handler
   - application service
   - domain model
   - port/repository
   - tests
2. Change the smallest set of layers that preserves the current architecture.
3. Reuse existing patterns for errors, DTOs, validation, and repository access.
4. Avoid creating “v2” paths or duplicate handlers unless there is a migration strategy.

## When creating a new feature

Create a new backend area only when the feature introduces a genuinely new bounded concern.

Before creating a new module, ask:

- Is this a new domain concept?
- Is this just a new use case over an existing concept?
- Can this be represented as a new node, service method, or repository method inside an existing area?

Usually:

- new use case -> extend existing application service
- new persistence behavior -> extend existing port + adapter
- new auth primitive -> add explicit node metadata + runtime worker + flow integration
- new HTTP capability -> add handler + service call + tests

## API and handler rules

- Handlers in `src/adapters/web` should validate/map requests and call application services.
- Avoid putting domain logic directly in handlers.
- Keep response shapes explicit and stable.
- Preserve current realm and auth middleware conventions.

## Error handling

- Use existing error types and validation patterns.
- Prefer returning structured validation or security errors over generic system failures.
- Do not swallow errors that should surface to observability or audit.

## Config and infra constraints

- ReAuth is SQLite-first and single-binary.
- Prefer local storage, SQLite tables, and in-process scheduling/workers where needed.
- Do not introduce new runtime infrastructure without a deliberate product decision.
