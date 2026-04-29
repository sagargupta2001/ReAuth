# ReAuth Agent Guide

This file is the root entrypoint for any agent working in this repository. Start here, then follow the deeper docs in `docs/agent/`.

## Read Order

1. `README.md`
2. `docs/agent/README.md`
3. `docs/agent/01-core-mental-models.md`
4. `docs/agent/02-repo-map.md`
5. The matching development guide:
   - Backend work: `docs/agent/03-backend-development.md`
   - Frontend work: `docs/agent/04-frontend-development.md`
   - Cross-cutting work: read both
6. `docs/agent/05-testing.md`
7. If the change touches persistence or seeds: `docs/agent/06-migrations-and-data.md`
8. `docs/agent/07-spec-driven-delivery.md`

Then load the relevant `docs/memory/` files for the area you are changing.

## Project Shape

- `src/`: Rust backend
- `ui/`: React + TypeScript + Vite frontend
- `migrations/`: SQLite schema and migration history
- `docs/agent/`: required implementation workflow
- `docs/memory/`: architecture and product memory
- `docs/specs/` and `specs/`: feature specs and templates

## Standard Commands

- Backend dev server: `make dev`
- Embedded UI build/run: `make embed`
- Rust format: `make fmt`
- Rust lint: `make clippy`
- Rust tests: `make test`
- Rust doc tests: `make test-docs`
- Rust coverage: `make coverage`
- UI install/build: `cd ui && npm install && npm run build`
- UI dev server: `cd ui && npm run dev`
- UI lint: `cd ui && npm run lint`
- UI tests: `cd ui && npm run test`
- UI coverage: `cd ui && npm run coverage`
- Full pre-PR validation: `make run-before-raising-pr`

## Working Rules

- Prefer specs for any meaningful feature or behavior change. Create or update the relevant file in `specs/`.
- Implement the smallest coherent vertical slice that satisfies the spec.
- Add or update tests with each change.
- If you change architecture, flows, or developer workflow, update the relevant `docs/memory/` or `docs/agent/` file.
- Treat `docs/agent/` as the execution workflow and `docs/memory/` as deeper system context.

## Runtime Notes

- Backend default URL: `http://127.0.0.1:3000`
- UI dev URL: `http://localhost:5173`
- Default database: `sqlite:data/reauth.db`
- Config defaults live in `config/default.toml`

## When In Doubt

- Prefer existing patterns over inventing new structure.
- Read the local docs before changing backend boundaries, auth flows, migrations, or UI architecture.
- Run the narrowest relevant checks first, then the broader validation target before handing work off.
