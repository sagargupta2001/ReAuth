# Feature Roadmap: Testing

## Goal
- Build a scalable, robust testing framework for the backend (first) and UI (second), with
  unit + integration + doc tests and a path to **100% coverage**.

## Current state (code-aligned)
- [x] Structured test framework in place (unit + integration + helpers).
- [x] Unit tests across domain + application layers.
- [x] Integration tests for API health/JWKS + SQLite repositories.
- [x] Coverage reporting (HTML via `cargo llvm-cov`).
- [x] Rust doc tests running in CI workflow.
- [ ] Coverage gate/threshold (currently disabled; planned to re-enable).

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
   - Add `TestDb` helper for repository-level tests.
2. **Unit tests (domain + application)**
   - Flow compiler/validator, RBAC permission resolution, auth/session validation.
   - Focus on pure logic and critical invariants.
   - **Done:** RBAC, auth/session, flow engine/executor, realm, OIDC services.
3. **Integration tests (API + persistence)**
   - **Done (initial):** API health/JWKS + SQLite repositories (RBAC/User/Realm/OIDC/Flow/FlowStore/AuthSession/RefreshToken).
   - **Next:** Auth login + OIDC start/finish, RBAC CRUD + permission assignment,
     group/role composites, flow publish/execute.
   - Use real DB + migrations for each suite; keep tests isolated via temp DBs.
4. **Rust doc tests**
   - Add runnable examples for public APIs and domain primitives.
5. **Coverage reporting**
   - `cargo llvm-cov` HTML reports enabled.
   - Re-enable a **60%** gate once baseline stabilizes; increase incrementally toward **100%**.
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

## Integration test plan (backend)
- **Auth/OIDC flows**
  - Start login, validate client, issue codes/tokens, refresh, revoke.
- **RBAC**
  - CRUD for roles/groups/custom permissions, assignments, composites, permission resolution.
- **Flows**
  - Draft create/update/publish, version/deployment queries, execution round trips.
- **Persistence guarantees**
  - Transactional paths (create/update/delete with tx), not-found handling, paging/sorting.
- **Fixtures**
  - Keep `TestDb` + `TestContext`, optional seed-on/off via env toggles.

## Later
- UI testing stack (Vitest + Testing Library + MSW).
- E2E flows (Playwright) for login + flow builder.
- Load/perf testing for large RBAC graphs and flow executions.

## Risks / dependencies
- Integration tests require stable fixtures and deterministic DB setup.
- Coverage gating may slow iteration until the baseline is established.

## Open questions
- Should we add a `test-support` feature for shared helpers, or keep helpers in `tests/support`?
