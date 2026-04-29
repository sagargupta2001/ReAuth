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

- `docs/agent/` tells you how to work in this codebase.
- `docs/memory/` gives deeper architectural and product context.
- `docs/specs/` is the build contract for an individual feature.

## Current Specs

- `passkey-first-auth.md`
- `magic-link-builtins.md`
- `spec-template.md`

## Rule

If a change materially affects business rules, APIs, domain models, flows, or persistence, it should have a spec.
