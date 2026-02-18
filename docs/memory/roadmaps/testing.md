# Feature Roadmap: Testing

## Goal
- Build a scalable, robust testing framework for the backend (first) and UI (second), with
  unit + integration + doc tests and a path to **100% coverage**.

## Current state (code-aligned)
- [ ] No structured test framework in place.
- [ ] No known unit or integration tests.
- [ ] No coverage reporting.
- [ ] Rust doc tests not used.

## Testing structure (Rust, open-source style)
- **Unit tests**: co-located `#[cfg(test)] mod tests` inside `crates/reauth_core/src/**`
  (mirrors hex layers: domain, application, adapters, bootstrap).
- **Integration tests**: `crates/reauth_core/tests/` as the primary harness.
  - Suggested folders: `api/`, `application/`, `persistence/`, `flow/`, `rbac/`, `auth/`.
- **Shared helpers/fixtures**: `crates/reauth_core/tests/support/` (or `src/test_support`
  behind `cfg(test)` / `test-support` feature).
- **Test data**: `crates/reauth_core/tests/fixtures/` (JSON/TOML for flows, users, realms).

## Plan (backend first, then UI)

### Phase 1: Backend foundation (Now)
1. **Test harness + fixtures**
   - Add `TestContext`/`TestApp` that boots Axum with an isolated SQLite DB.
   - Run migrations on setup; use per-test temp DB files.
2. **Unit tests (domain + application)**
   - Flow compiler/validator, RBAC permission resolution, auth/session validation.
   - Focus on pure logic and critical invariants.
3. **Integration tests (API + persistence)**
   - Auth login + OIDC start/finish, RBAC CRUD + permission assignment,
     group/role composites, flow publish/execute.
   - Use real DB + migrations for each test suite.
4. **Rust doc tests**
   - Add runnable examples for public APIs and domain primitives.
5. **Coverage reporting**
   - Adopt `cargo llvm-cov` (LCOV/HTML) and set an initial gate at **60%**,
     with a staged plan to raise to **100%**.
6. **Developer workflow**
   - Add `Makefile` target `run-before-raising-pr` to run fmt, clippy, unit tests,
     integration tests, and doc tests (plus coverage).

### Phase 2: Backend expansion (Next)
1. **Adapter/port contract tests**
   - Ensure repository implementations satisfy port contracts.
2. **Property-based tests**
   - RBAC resolution and flow graph invariants (cycle prevention, single start node).
3. **Coverage hardening**
   - Raise coverage gates incrementally until **100%** is enforced.

## Later
- UI testing stack (Vitest + Testing Library + MSW).
- E2E flows (Playwright) for login + flow builder.
- Load/perf testing for large RBAC graphs and flow executions.

## Risks / dependencies
- Integration tests require stable fixtures and deterministic DB setup.
- Coverage gating may slow iteration until the baseline is established.

## Open questions
- Should we add a `test-support` feature for shared helpers, or keep helpers in `tests/support`?
