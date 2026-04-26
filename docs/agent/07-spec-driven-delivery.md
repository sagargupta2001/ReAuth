# Spec-Driven Delivery

Every meaningful feature should start from a spec in `specs/`.

## Why

Specs are the source of truth for:

- scope
- business rules
- domain impact
- API impact
- required tests
- out-of-scope boundaries

They prevent implementation from drifting into undocumented product decisions.

## When a spec is required

Create or update a spec when:

- you are building a new feature
- you are changing business rules
- you are changing domain models or persistence
- you are adding or modifying public HTTP endpoints
- you are adding new auth nodes, flow behavior, or protocol behavior

A tiny refactor or a local bug fix may not need a new spec, but it should still align with an existing one where applicable.

## Workflow

1. Identify whether an existing spec already covers the work.
2. If not, create a new spec from `specs/spec-template.md`.
3. Mark it `Draft` while the shape is still being clarified.
4. Move it to `Ready` once scope, rules, and acceptance are stable enough to build.
5. Implement against the spec.
6. Mark it `Implemented` when the code and required tests are done.

## How to name specs

Use concise kebab-case filenames:

- `specs/passkey-foundation.md`
- `specs/magic-link-login.md`
- `specs/realm-security-headers.md`

## Guidance for writing a good spec

- Keep business rules independently testable.
- Separate new entities from modified entities.
- Be explicit about module impact.
- Call out edge cases early.
- Keep out-of-scope items concrete so they actually constrain implementation.
- Add test scenarios that map directly to acceptance.

## Modifying an existing feature vs creating a new one

### Modify an existing feature when

- the user-facing capability is the same
- the same domain concept is being extended
- the same module owns the behavior already
- the schema/API changes are evolutionary

In this case, update the existing spec if one exists.

### Create a new feature spec when

- a new user-facing capability is being introduced
- a new auth method, protocol primitive, or flow primitive is added
- a new domain concept appears
- a change would otherwise overload an unrelated existing spec

## Relationship to roadmap docs

- `docs/memory/roadmaps/` explains strategic direction.
- `specs/` explains implementation-ready feature scope.

Roadmaps are broad and evolving.
Specs should be narrower and directly buildable.
