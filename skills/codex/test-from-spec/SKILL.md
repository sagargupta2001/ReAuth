---
name: test-from-spec
description: Use when a ReAuth feature spec already exists and you need to add, tighten, or verify tests against that spec. This skill maps the spec's test scenarios to the current implementation, uses Graphify before broad architecture scanning when graphify-out/graph.json exists, and adds the smallest effective tests at the right layer.
---

# Test From Spec

## Overview

Add or verify test coverage directly from a ReAuth spec. Use this after or during implementation when the code exists but the acceptance coverage is incomplete or unclear.

## Workflow

1. Read `AGENTS.md`, `docs/agent/05-testing.md`, `docs/agent/07-spec-driven-delivery.md`, and the target spec in `docs/specs/`.
2. Use the spec's `Test Scenarios` as the expected behavior contract.
3. Inspect the current implementation and existing tests before writing new ones.
4. If architecture or codebase context is needed and `graphify-out/graph.json` exists:
   - compare the commit recorded in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
5. Choose the smallest effective layer:
   - unit for domain or service rules
   - integration for API, repository, and cross-module behavior
   - UI tests for feature flows, forms, and route-driven behavior
6. Add or update only the missing coverage needed by the spec.
7. Run `make docs-check` and the narrowest relevant verification commands.

## Coverage Expectations

- Happy path.
- Validation failure.
- Business rule edge case.
- Failure or recovery path when the spec defines one.

## Guardrails

- Do not invent behaviors that are not in the spec.
- If code and spec conflict, report the gap instead of hiding it behind tests.
- Prefer adding the smallest effective tests over broad suite rewrites.
