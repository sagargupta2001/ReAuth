# Testing Guide

Testing is required for meaningful changes.

## Default expectation

- Every non-trivial change should add or update tests.
- Prefer the smallest test level that gives strong confidence.
- Keep tests close to the code unless the test is cross-module or infrastructure-heavy.

## Backend testing approach

- Small/medium unit tests:
  - inline `#[cfg(test)] mod tests`
  - or sibling `tests.rs` for larger module-level suites
- Integration tests:
  - `tests/` for API, SQLite, and cross-module behavior
- Mock trait-based dependencies with `mockall` where appropriate

Use backend tests for:

- domain rules
- application orchestration
- repository behavior
- handler-level API behavior
- flow execution invariants

## Frontend testing approach

- Use Vitest for unit and component tests
- Use Testing Library for component behavior
- Use MSW for network isolation
- Keep tests near the feature/component where practical

Use frontend tests for:

- feature hooks
- form validation behavior
- route- or state-driven rendering
- API hook behavior and optimistic/invalidation flows

## What to test for a feature

At minimum, cover:

1. Happy path
2. Validation failure
3. Business rule edge case
4. Failure or recovery path

## Verification workflow

Preferred full verification before handoff:

```bash
make run-before-raising-pr
```

That is the project-standard pre-PR command.

Targeted commands when iterating:

```bash
cargo test --all-features
cargo test --doc
cd ui && npm run test
cd ui && npm run lint
```

## Testing and specs

- Every feature spec in `specs/` should include explicit test scenarios.
- Before marking a spec implemented, ensure those scenarios are covered by code or documented manual verification when automation is not yet feasible.
