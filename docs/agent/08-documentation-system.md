# Documentation System

Use this guide when the task changes docs, developer workflow, or project memory.

## Canonical Roles

- `AGENTS.md` is the only onboarding contract. Keep read order, repo commands, and Graphify-first rules there.
- `docs/README.md` is the stable docs map. Keep it short and navigational.
- `docs/agent/` holds imperative guidance: how to work, what to verify, and where to put changes.
- `docs/memory/` holds descriptive knowledge: architecture, domain behavior, subsystem notes, ADRs, and roadmaps.
- `docs/specs/` holds feature-specific contracts: scope, rules, interface impact, and test scenarios.

## Update Triggers

- Update `AGENTS.md` when the repo-wide operating contract changes.
- Update `docs/agent/` when workflow, testing guidance, or docs governance changes.
- Update `docs/memory/` when durable architecture, domain concepts, or subsystem behavior changes.
- Update `docs/memory/adr/` when a significant decision is locked.
- Update `docs/memory/roadmaps/` when strategic sequencing, milestones, or dependencies change.
- Update `docs/specs/` when a feature's scope, rules, or acceptance changes.
- Update the relevant README indexes when files are added, removed, or renamed.

## Anti-Duplication Rules

- Keep onboarding order only in `AGENTS.md`.
- Keep folder README files thin. They should map the directory, not restate the full operating contract.
- Link to deeper docs instead of copying architecture or product summaries into implementation guides.
- Keep examples in index files limited to real files that exist in the repo.
- Prefer one canonical statement per rule. If a rule already belongs in `AGENTS.md`, other docs should point there.

## Graphify-First Workflow

- For architecture or codebase-structure questions, run `graphify query`, `graphify explain`, or `graphify path` first when `graphify-out/graph.json` exists.
- Compare the commit recorded in `graphify-out/GRAPH_REPORT.md` to `git rev-parse HEAD` before treating the graph as current.
- If the graph is missing, stale, or insufficient, fall back to targeted repo reads instead of broad blind scanning.
- After code changes, run `graphify update .` so the graph stays aligned with the workspace.

## Validation

- Run `make docs-check` after changing docs structure, indexes, or repo-local agent guidance.
- Treat broken local doc references, stale indexes, and duplicated entrypoint language as validation failures.
