# Feature Roadmap: Testing

## Goal
- Establish reliable unit + integration test coverage for core auth, RBAC, and API flows.

## Current state (code-aligned)
- [ ] Unit test baseline is minimal or unknown.
- [ ] Integration tests for API endpoints are minimal or unknown.
- [ ] Coverage reporting is not enforced.

## MVP scope (prioritized)
1. **Unit testing foundation**
   - Add test harness for core domain logic (RBAC, roles, groups, permissions).
   - Target high-value pure functions and critical validation paths.
2. **Integration testing for APIs**
   - Add API tests for auth, RBAC, group management, permissions, composites.
   - Use real DB (sqlite) with migrations in test setup.
3. **Coverage targets**
   - Establish baseline coverage thresholds and report in CI.
   - Track coverage for RBAC and auth flows first.

## Later
- Load testing for large RBAC graphs and group trees.
- Property-based testing for permission resolution and cycle prevention.

## Risks / dependencies
- Integration tests require stable test fixtures and repeatable DB setup.

## Open questions
- What minimum coverage target is acceptable for the first enforced threshold?
