Derive and add missing test coverage from this ReAuth spec or feature reference: `$ARGUMENTS`.

Workflow:

1. Read `AGENTS.md`, `docs/agent/05-testing.md`, `docs/agent/07-spec-driven-delivery.md`, and the target spec in `docs/specs/`.
2. Use the spec's `Test Scenarios` as the source of truth for expected behavior.
3. Inspect the current implementation and existing tests before adding anything.
4. If architecture or codebase understanding is needed and `graphify-out/graph.json` exists, use Graphify first:
   - compare the commit in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
5. Add the smallest effective tests at the right layer:
   - unit for domain or service rules
   - integration for API, repository, and cross-module behavior
   - UI tests for feature flows, forms, and route-driven behavior
6. Run `make docs-check` and the narrowest relevant test commands first.

Rules:

- Cover happy path, validation failure, business edge case, and failure or recovery path when the spec requires them.
- Do not invent behaviors that are not in the spec.
- If the code and spec conflict, report the gap clearly instead of papering over it with tests.

Deliverable:

- The tests added or updated.
- What scenarios are now covered.
- Any remaining coverage gaps caused by incomplete implementation or ambiguous spec behavior.
