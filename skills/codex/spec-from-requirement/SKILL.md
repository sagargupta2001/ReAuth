---
name: spec-from-requirement
description: Use when a user has a rough feature requirement, ticket, or product note and wants a ReAuth spec created or updated under docs/specs. This skill turns ambiguous requirements into implementation-ready specs by reading the repo docs, checking for existing specs first, and using Graphify before broad architecture scanning when graphify-out/graph.json exists.
---

# Spec From Requirement

## Overview

Turn a rough requirement into a buildable ReAuth spec. Use this before meaningful implementation work when the request changes business rules, public APIs, persistence, flows, or auth behavior.

## Workflow

1. Read `AGENTS.md`, `docs/README.md`, `docs/agent/07-spec-driven-delivery.md`, and `docs/agent/08-documentation-system.md`.
2. Check `docs/specs/` first to decide whether to update an existing spec or create a new one.
3. If architecture or codebase context is needed and `graphify-out/graph.json` exists:
   - compare the commit recorded in `graphify-out/GRAPH_REPORT.md` with `git rev-parse HEAD`
   - run `graphify query`, `graphify explain`, or `graphify path` before broad repo scanning
4. Read only the relevant `docs/memory/` files for the affected area.
5. Write or update the spec in `docs/specs/` using `docs/specs/spec-template.md`.
6. Keep the spec status accurate:
   - `Draft` when product or scope questions are still open
   - `Ready` when the feature is specific enough to implement

## Required Output

- Explicit business rules that are independently testable.
- Concrete module impact across backend, frontend, persistence, and flows when relevant.
- API and persistence changes only when the requirement actually needs them.
- Test scenarios that can be mapped directly to implementation checks.
- Clear out-of-scope boundaries.
- Open questions instead of invented product policy when intent is still unresolved.

## Guardrails

- Do not implement code in this skill.
- Do not create a new spec when an existing spec should clearly be updated.
- For auth and security features, explicitly cover system prerequisites, realm policy, and flow composition.
- If the requirement is too vague to make a product decision, preserve the ambiguity in `Open Questions`.
