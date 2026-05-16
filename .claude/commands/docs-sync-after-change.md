Review the current ReAuth changes and sync the affected specs, memory docs, indexes, and graph state. Additional context: `$ARGUMENTS`.

Workflow:

1. Read `AGENTS.md`, `docs/README.md`, and `docs/agent/08-documentation-system.md`.
2. Inspect the current diff to determine whether the changes affect:
   - `docs/specs/`
   - `docs/memory/`
   - `docs/agent/`
   - README or doc indexes
3. If architecture or codebase understanding is needed and `graphify-out/graph.json` exists, use Graphify first:
   - compare the commit in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Update the required docs only where the change created durable drift.
5. Update any affected README indexes.
6. Run `make docs-check`.
7. If code changed, run `graphify update .`.

Rules:

- Do not rewrite unrelated docs just because you touched nearby code.
- Keep folder READMEs thin and navigational.
- Link instead of duplicating architecture or workflow guidance.
- If a code change should have a spec but does not, call that out explicitly.

Deliverable:

- The docs or indexes updated.
- `make docs-check` result.
- Whether Graphify was refreshed.
