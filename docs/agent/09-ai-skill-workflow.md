# AI Skill Workflow

Use this guide when you want Claude or Codex to help turn a requirement into a spec, implement that spec, and tighten the tests.

## What Exists

- Claude project commands live in `.claude/commands/`.
- Codex skill bundles live in `skills/codex/`.
- The current bundles are:
  - `spec-from-requirement`
  - `implement-from-spec`
  - `test-from-spec`
  - `docs-sync-after-change`

## Claude Workflow

Claude can use the project commands immediately.

Commands:

- `/spec-from-requirement <feature or requirement summary>`
- `/implement-from-spec <docs/specs/...>`
- `/test-from-spec <docs/specs/...>`
- `/docs-sync-after-change <optional context>`

Recommended flow for a feature:

1. Start with `/spec-from-requirement`.
2. Review the generated spec and tighten any open questions.
3. Run `/implement-from-spec` with the final spec path.
4. Run `/test-from-spec` against the same spec path.
5. Run `/docs-sync-after-change` if the implementation changed docs, indexes, or memory.

## Codex Workflow

The Codex bundles in `skills/codex/` are source-controlled project skills. To make them available as user skills, install them into `${CODEX_HOME:-$HOME/.codex}/skills`.

Install command:

```bash
bash scripts/install_codex_skills.sh
```

After that, invoke them by name in your prompt, for example:

- `Use the spec-from-requirement skill for this feature: add invite-based user onboarding by email.`
- `Use the implement-from-spec skill for docs/specs/user-invitations.md.`
- `Use the test-from-spec skill for docs/specs/user-invitations.md.`
- `Use the docs-sync-after-change skill after these changes.`

## Exact Feature Workflow

For a typical feature, use this order:

1. Requirement to spec.
   - Goal: convert a rough ask into a buildable spec in `docs/specs/`.
   - Tool: `spec-from-requirement`
2. Spec review.
   - Goal: decide any open questions before code starts.
   - Human step: review `Business Rules`, `Module Impact`, `API Changes`, `Persistence Changes`, and `Test Scenarios`.
3. Spec to implementation.
   - Goal: implement the smallest coherent vertical slice from the spec.
   - Tool: `implement-from-spec`
4. Spec to tests.
   - Goal: add or tighten tests directly from the spec's acceptance scenarios.
   - Tool: `test-from-spec`
5. Docs and memory sync.
   - Goal: update stale indexes, specs, memory, or graph state after the implementation.
   - Tool: `docs-sync-after-change`

## Expectations

- All skills should read `AGENTS.md` first.
- For architecture or codebase questions, all skills should use Graphify first when `graphify-out/graph.json` exists.
- `spec-from-requirement` should not implement code.
- `implement-from-spec` should stop if the spec is still too vague.
- `test-from-spec` should use the spec's `Test Scenarios` as the behavior contract.
- `docs-sync-after-change` should run `make docs-check` and refresh Graphify when code changed.
