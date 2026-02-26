# Feature Roadmap: Testing

## Goal
- Build a scalable, robust testing framework for the backend and UI, with unit + integration + doc tests and a path to **100% coverage**.

## Current state (code-aligned)
- [x] Structured test framework in place (unit + integration + helpers).
- [x] Unit tests across domain + application layers.
- [x] Integration tests for API health/JWKS + SQLite repositories.
- [x] Coverage reporting (HTML via `cargo llvm-cov`).
- [x] Rust doc tests running in CI workflow.
- [x] **UI Testing Stack**: Vitest + Testing Library + MSW implemented.
- [x] **Automated CI**: GitHub Actions running all checks on PRs.
- [x] **Standardized Structure**: Co-located tests (inline or sibling files) for scalability.
- [x] **Optimized Speed**: Reduced password hashing cost in test mode to speed up the suite.

## Testing structure

### Backend (Rust)
- **Small/Medium Suites**: Co-located `#[cfg(test)] mod tests` inside the source file.
- **Large Suites**: Sibling `tests.rs` file within a folder module (e.g., `application/rbac_service/tests.rs`).
- **Integration Tests**: `crates/reauth_core/tests/` for multi-module and infrastructure tests (SQLite, API).
- **Mocks**: Using `mockall` for trait-based dependency mocking.

### UI (React + TypeScript)
- **Unit/Component Tests**: Sibling `.test.tsx` files (e.g., `button.test.tsx`).
- **API Mocking**: MSW (Mock Service Worker) for network-level isolation.
- **Test Runner**: Vitest for speed and Vite compatibility.

## Developer Workflow

The project uses a `Makefile` to unify verification across the stack.

### Pre-PR Verification
Run the following command before pushing any changes:
```bash
make run-before-raising-pr
```
This target performs:
1.  **Rust Formatting** (`cargo fmt`)
2.  **Rust Linting** (`cargo clippy`)
3.  **Backend Tests** (`cargo test`)
4.  **Documentation Tests** (`cargo test --doc`)
5.  **Coverage Analysis** (`cargo llvm-cov`)
6.  **UI Linting** (`npm run lint`)
7.  **UI Testing** (`npm run test`)

### GitHub Actions
The same checks are enforced in GitHub. A PR cannot be merged unless the **Build & Validate** job passes.

## Plan

### Phase 1: Foundation (Completed)
- [x] Setup unified `Makefile` and CI.
- [x] Implement initial unit/integration tests for all major services.
- [x] Setup UI testing environment.

### Phase 2: Expansion (Next)
1. **[x] Adapter/port contract tests**
   - Ensure repository implementations satisfy port contracts.
2. **[ ] E2E Testing**
   - Implement Playwright/Cypress for critical user journeys (Login, Flow Builder).
3. **[x] Property-based tests**
   - RBAC resolution and flow graph invariants.
4. **[ ] Coverage hardening**
   - Raise coverage gates incrementally toward **100%**.

## Risks / dependencies
- Integration tests require stable fixtures and deterministic DB setup.
- E2E tests can be brittle and require a stable backend environment.
