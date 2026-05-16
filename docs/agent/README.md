# Agent Docs

Use this folder as a guide map after reading `AGENTS.md`.

## Guides

- `docs/agent/01-core-mental-models.md`
- `docs/agent/02-repo-map.md`
- `docs/agent/03-backend-development.md`
- `docs/agent/04-frontend-development.md`
- `docs/agent/05-testing.md`
- `docs/agent/06-migrations-and-data.md`
- `docs/agent/07-spec-driven-delivery.md`
- `docs/agent/08-documentation-system.md`
- `docs/agent/09-ai-skill-workflow.md`

## Typical Task Paths

- Backend changes: `docs/agent/01-core-mental-models.md`, `docs/agent/02-repo-map.md`, `docs/agent/03-backend-development.md`, `docs/agent/05-testing.md`, and `docs/agent/06-migrations-and-data.md` when persistence changes.
- Frontend changes: `docs/agent/01-core-mental-models.md`, `docs/agent/02-repo-map.md`, `docs/agent/04-frontend-development.md`, and `docs/agent/05-testing.md`.
- Cross-cutting features: both development guides plus `docs/agent/07-spec-driven-delivery.md`.
- Docs or workflow changes: `docs/agent/08-documentation-system.md` plus the relevant `docs/memory/` or `docs/specs/` index.
- AI-assisted requirement-to-feature flow: `docs/agent/09-ai-skill-workflow.md`.

## Project Memory

- Start with `docs/memory/README.md`.
- Pull in the specific subsystem docs before implementation.
- Use `docs/memory/roadmaps/README.md` and `docs/memory/adr/README.md` when the task touches strategic direction or prior decisions.

## Graphify

- For architecture or codebase-structure questions, run `graphify query`, `graphify explain`, or `graphify path` first when `graphify-out/graph.json` exists.
- If the graph is missing, stale, or insufficient, fall back to targeted repo reads.
