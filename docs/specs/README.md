# Specs

This folder holds implementation-facing feature specs.

## Purpose

Each spec should define:

- who the feature is for
- business rules
- domain changes
- module and API impact
- required test scenarios
- out-of-scope boundaries
- open questions

## Workflow

1. Copy `spec-template.md` to a new kebab-case feature file.
2. Fill in the feature scope before implementation.
3. Keep the spec updated as decisions become concrete.
4. Mark the status:
   - `Draft`
   - `Ready`
   - `Implemented`

## Relationship to other docs

- `AGENTS.md` is the repo-wide onboarding contract.
- `docs/agent/07-spec-driven-delivery.md` explains how to work with specs in this codebase.
- `docs/memory/` gives deeper architectural and product context.
- `docs/specs/` is the build contract for an individual feature.

## Index

- `magic-link-builtins.md`
- `passkey-first-auth.md`
- `spec-template.md`
- `user-invitations.md`
- `users-table-filters.md`

## Rule

If a change materially affects business rules, APIs, domain models, flows, or persistence, it should have a spec.
