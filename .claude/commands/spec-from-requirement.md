Create or update a ReAuth feature spec from the current conversation and these additional arguments: `$ARGUMENTS`.

Workflow:

1. Read `AGENTS.md`, `docs/README.md`, `docs/agent/07-spec-driven-delivery.md`, and `docs/agent/08-documentation-system.md`.
2. Check `docs/specs/` first to decide whether this should update an existing spec or create a new one.
3. If architecture or codebase understanding is needed and `graphify-out/graph.json` exists, use Graphify first:
   - compare the commit in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Read only the relevant `docs/memory/` files for the affected area.
5. Write or update the spec in `docs/specs/` using `docs/specs/spec-template.md`.

Rules:

- Do not implement code in this command.
- Keep business rules explicit and independently testable.
- Be concrete about module impact, API impact, persistence impact, and test scenarios.
- For auth and security features, explicitly cover system prerequisites, realm policy, and flow composition.
- If product intent is unclear, keep the uncertainty in `Open Questions` instead of inventing policy.
- Prefer `Draft` when scope is still moving and `Ready` when the spec is buildable.

Deliverable:

- The updated spec path.
- Any open questions or assumptions that still need human confirmation.
