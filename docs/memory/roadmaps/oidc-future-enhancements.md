# Feature Roadmap: OIDC Future Enhancements

## Goal
- Extend OIDC compliance, interoperability, and security beyond the Phase 1 MVP baseline.

## Current state
- Phase 1 OIDC hardening is complete (see `oidc-flow-engine.md`).

## Next
- Add OIDC compliance tests (conformance harness).
- Add structured OIDC error responses aligned with the spec.
- Add clock-skew handling and auditing for token reuse detection.

## Later
- Optional DPoP or MTLS support for high-security deployments.
- `/userinfo` scope-aware claim filtering (e.g., `profile`, `email`).
- Signed request objects (JAR) and pushed authorization requests (PAR).

## Risks / dependencies
- Conformance tests require stable test fixtures and token issuance semantics.
- Advanced features like DPoP/MTLS depend on client capability detection and new config.

## Open questions
- Which conformance harness to standardize on for CI?
- Should advanced features be opt-in per realm or global config?
