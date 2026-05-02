# Spec-Driven Delivery

Every meaningful feature should start from a spec in `docs/specs/`.

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
2. If not, create a new spec from `docs/specs/spec-template.md`.
3. Mark it `Draft` while the shape is still being clarified.
4. Move it to `Ready` once scope, rules, and acceptance are stable enough to build.
5. Implement against the spec.
6. Mark it `Implemented` when the code and required tests are done.

## How to name specs

Use concise kebab-case filenames:

- `docs/specs/passkey-first-auth.md`
- `docs/specs/magic-link-builtins.md`
- `docs/specs/realm-security-headers.md`

## Guidance for writing a good spec

- Keep business rules independently testable.
- Separate new entities from modified entities.
- Be explicit about module impact.
- For auth/security features, document capability scope explicitly:
  - system/operator prerequisites
  - realm policy
  - flow composition
- Call out edge cases early.
- Keep out-of-scope items concrete so they actually constrain implementation.
- Add test scenarios that map directly to acceptance.

## Auth Feature Completion Checklist

For any new auth capability (node, protocol step, or journey), the spec and implementation must cover all items below before marking `Implemented`:

1. Runtime + flow node
- Node provider metadata (`id`, `outputs`, `config_schema`, `default_template_key`)
- Runtime worker wiring and execution paths (`execute`, `handle_input`, fallback/error paths)

2. Flow-builder UX
- Node appears with normal authenticator styling in canvas (node type mapping)
- Recommended flow presets include explicit edges between new node and existing password/terminal nodes
- Publish-time validation covers capability/policy prerequisites

3. Public/API surface
- Required options/verify endpoints (or equivalent) documented and tested
- Session continuity across multi-step endpoints (do not rely on fragile UI-only state)

4. Theme/Fluid surface
- Dedicated system page key(s) for the auth feature (no implicit reuse unless intentional and documented)
- Default page blueprint(s) added so theme editor exposes those pages immediately
- Template-key mapping updated across executor + theme handlers

5. Operator experience
- Realm settings/presets for enabling and applying recommended flows
- Observability diagnostics for success/failure/replay/suspicious states

6. Tests
- Happy path and fallback path
- Failure path(s), including signature/challenge mismatch where relevant
- Integration test for admin preset endpoints and deployed graph assertions

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
- `docs/specs/` explains implementation-ready feature scope.

Roadmaps are broad and evolving.
Specs should be narrower and directly buildable.
