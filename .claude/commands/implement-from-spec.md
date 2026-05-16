Implement the requested ReAuth feature from this spec path or feature reference: `$ARGUMENTS`.

Workflow:

1. Read `AGENTS.md`, `docs/README.md`, `docs/agent/02-repo-map.md`, the matching backend/frontend development guide, `docs/agent/05-testing.md`, and `docs/agent/08-documentation-system.md`.
2. Read the target spec in `docs/specs/` and confirm whether it is specific enough to implement.
3. If architecture or codebase understanding is needed and `graphify-out/graph.json` exists, use Graphify first:
   - compare the commit in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Read only the relevant `docs/memory/` files for the feature area.
5. Implement the smallest coherent vertical slice that satisfies the spec.
6. Add or update the relevant tests.
7. Update docs or memory only if architecture, workflow, or durable behavior changed.
8. Run `make docs-check` and the narrowest relevant verification commands before broader validation.

Rules:

- If the spec is too vague or still making product decisions, stop and tighten the spec first.
- Preserve existing architectural boundaries from `docs/agent/02-repo-map.md`.
- Do not create parallel v2 paths or new top-level modules unless the spec requires it.
- Keep the implementation scoped to the requested slice rather than trying to finish adjacent roadmap work.

Deliverable:

- Implemented code and tests.
- The commands run for verification.
- Any spec gaps or follow-up work discovered during implementation.
