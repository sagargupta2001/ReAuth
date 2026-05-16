---
name: implement-from-spec
description: Use when a ReAuth feature already has a spec in docs/specs and the next step is implementation. This skill reads the spec, the matching agent and memory docs, uses Graphify before broad architecture scanning when graphify-out/graph.json exists, and implements the smallest coherent vertical slice with tests.
---

# Implement From Spec

## Overview

Implement a ReAuth feature from an existing spec. Use this only when the spec is specific enough to build against.

## Workflow

1. Read `AGENTS.md`, `docs/README.md`, `docs/agent/02-repo-map.md`, the relevant development guide, `docs/agent/05-testing.md`, and `docs/agent/08-documentation-system.md`.
2. Read the target spec in `docs/specs/`.
3. If architecture or codebase context is needed and `graphify-out/graph.json` exists:
   - compare the commit recorded in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Read only the relevant `docs/memory/` files for the area being changed.
5. Map the smallest coherent vertical slice before editing:
   - backend: handler, service, domain, port, adapter, migration, tests
   - frontend: page, widget, feature, entity, shared API, tests
6. Implement the slice without expanding into adjacent roadmap work.
7. Add or update tests for the slice.
8. Update docs or memory only when architecture, workflow, or durable behavior changed.
9. Run `make docs-check` and the narrowest relevant verification commands.

## Preconditions

- The feature has a spec path in `docs/specs/`.
- The spec is specific enough to implement. If it is still making product decisions, tighten the spec first.

## Guardrails

- Preserve the boundaries in `docs/agent/02-repo-map.md`.
- Do not create parallel v2 code paths unless the spec explicitly requires them.
- Do not silently broaden the spec. Record any discovered gaps instead.
- Favor the smallest coherent vertical slice over speculative completeness.
