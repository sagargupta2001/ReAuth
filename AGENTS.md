# ReAuth Agent Guide

This file is the root entrypoint for any agent working in this repository. Start here, then use `docs/README.md` and `docs/agent/README.md` as navigation only.

## Read Order

1. `README.md`
2. `docs/README.md`
3. `docs/agent/README.md`
4. `docs/agent/01-core-mental-models.md`
5. `docs/agent/02-repo-map.md`
6. The matching development guide:
   - Backend work: `docs/agent/03-backend-development.md`
   - Frontend work: `docs/agent/04-frontend-development.md`
   - Cross-cutting work: read both
7. `docs/agent/05-testing.md`
8. If the change touches persistence or seeds: `docs/agent/06-migrations-and-data.md`
9. `docs/agent/07-spec-driven-delivery.md`
10. If the task changes docs, workflow, or project memory: `docs/agent/08-documentation-system.md`
11. If you are using the repo's AI-assisted feature workflow: `docs/agent/09-ai-skill-workflow.md`

Then load the relevant `docs/memory/` files for the area you are changing.

## Project Shape

- `src/`: Rust backend
- `ui/`: React + TypeScript + Vite frontend
- `migrations/`: SQLite schema and migration history
- `docs/`: documentation system map and indexes
- `.claude/commands/`: project Claude slash commands
- `docs/agent/`: required implementation workflow
- `docs/memory/`: architecture and product memory
- `docs/specs/`: feature specs and templates
- `skills/codex/`: source-controlled Codex skill bundles for this repo

## Standard Commands

- Backend dev server: `make dev`
- Embedded UI build/run: `make embed`
- Rust format: `make fmt`
- Rust lint: `make clippy`
- Rust tests: `make test`
- Rust doc tests: `make test-docs`
- Rust coverage: `make coverage`
- Docs validation: `make docs-check`
- UI install/build: `cd ui && npm install && npm run build`
- UI dev server: `cd ui && npm run dev`
- UI lint: `cd ui && npm run lint`
- UI tests: `cd ui && npm run test`
- UI coverage: `cd ui && npm run coverage`
- Full pre-PR validation: `make run-before-raising-pr`

## Working Rules

- Prefer specs for any meaningful feature or behavior change. Create or update the relevant file in `docs/specs/`.
- Implement the smallest coherent vertical slice that satisfies the spec.
- Add or update tests with each change.
- If you change architecture, flows, or developer workflow, update the relevant `docs/memory/` or `docs/agent/` file.
- Treat `docs/agent/` as the execution workflow, `docs/memory/` as deeper system context, and `docs/specs/` as feature-level build contracts.

## Runtime Notes

- Backend default URL: `http://127.0.0.1:3000`
- UI dev URL: `http://localhost:5173`
- Default database: `sqlite:data/reauth.db`
- Config defaults live in `config/default.toml`

## When In Doubt

- Prefer existing patterns over inventing new structure.
- Read the local docs before changing backend boundaries, auth flows, migrations, or UI architecture.
- Run the narrowest relevant checks first, then the broader validation target before handing work off.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, invoke the `skill` tool with `skill: "graphify"` before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Before relying on the graph for architecture or codebase-structure answers, compare the commit recorded in `graphify-out/GRAPH_REPORT.md` to `git rev-parse HEAD`.
- If the graph is missing, stale, or insufficient for the question, fall back to targeted repo reads instead of broad blind scanning.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
