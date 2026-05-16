---
name: docs-sync-after-change
description: Use after ReAuth code or docs changes to keep specs, memory docs, indexes, and Graphify state aligned. This skill inspects the diff, applies the update rules from docs/agent/08-documentation-system.md, uses Graphify before broad architecture scanning when graphify-out/graph.json exists, and runs docs validation.
---

# Docs Sync After Change

## Overview

Synchronize the documentation system after a change. Use this when a feature, workflow, or architecture change may have left `docs/specs/`, `docs/memory/`, indexes, or graph state stale.

## Workflow

1. Read `AGENTS.md`, `docs/README.md`, and `docs/agent/08-documentation-system.md`.
2. Inspect the current diff to determine whether it affects:
   - `docs/specs/`
   - `docs/memory/`
   - `docs/agent/`
   - README or index files
3. If architecture or codebase context is needed and `graphify-out/graph.json` exists:
   - compare the commit recorded in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Update only the docs that the change actually invalidated.
5. Refresh any affected README indexes.
6. Run `make docs-check`.
7. If code changed, run `graphify update .`.

## Guardrails

- Do not rewrite unrelated docs.
- Keep folder READMEs thin and navigational.
- Link instead of duplicating architecture or workflow statements that already have a canonical home.
- If a code change should have a spec but does not, call that out explicitly.
