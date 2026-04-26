# Agent Docs

Purpose: this is the required entrypoint for any human or agent starting work in ReAuth.

This folder does not replace `docs/memory/`. It curates the minimum docs, rules, and workflow needed to start implementation consistently.

## Read this before any task

1. Read the root `README.md`.
2. Read `docs/agent/01-core-mental-models.md`.
3. Read `docs/agent/02-repo-map.md`.
4. Read the development guide that matches your change:
   - Backend-heavy: `docs/agent/03-backend-development.md`
   - Frontend-heavy: `docs/agent/04-frontend-development.md`
   - Cross-cutting: read both
5. Read `docs/agent/05-testing.md`.
6. If the task touches persistence, read `docs/agent/06-migrations-and-data.md`.
7. Read `docs/agent/07-spec-driven-delivery.md`.

## Then read task-specific project memory

Always pull in the relevant `docs/memory/` docs before implementation. Minimum project context:

- `docs/memory/00-context.md`
- `docs/memory/01-architecture.md`
- `docs/memory/03-control-flow.md`
- `docs/memory/09-roadmap.md`

Area-specific examples:

- Auth and login work:
  - `docs/memory/04-oidc-sso-flows.md`
  - `docs/memory/11-auth-flow-catalog.md`
- Flow builder and nodes:
  - `docs/memory/05-flow-builder.md`
  - relevant roadmap under `docs/memory/roadmaps/`
- UI/theme/fluid work:
  - `docs/memory/18-fluid-theme-builder.md`
  - `docs/memory/20-ui-development-practices.md`
- Data/model work:
  - `docs/memory/02-domain-model.md`
  - `docs/memory/07-data-model.md`

## Required workflow

For new features or meaningful changes:

1. Create or update a spec in `specs/`.
2. Align the implementation plan with the spec.
3. Implement the smallest coherent vertical slice.
4. Add or update tests with the change.
5. Update the spec status and any relevant memory docs if the architecture or workflow changed.

## Source of truth

- `docs/agent/` is the implementation workflow and engineering-rules entrypoint.
- `docs/memory/` is the deeper architectural and product memory.
- `specs/` is the source of truth for feature-level implementation scope and acceptance.
