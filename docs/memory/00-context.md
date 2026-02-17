# Context

## One-liner
ReAuth is a single-binary, self-hosted IdP inspired by Keycloak, built with Rust (backend) and React (UI), providing multi-realm auth, OIDC/SSO, RBAC, a flow builder, and a plugin POC.

## Goals
- Single-binary deployment option (Rust backend with embedded UI).
- Multi-realm identity and authorization management.
- Extensible auth flows via a visual flow builder.
- Basic RBAC with clear, auditable policies.
- Plugin extension via gRPC (POC).
- Developer-friendly local setup (SQLite, minimal infra).
- High performance with minimal footprint and fast startup.

## Non-goals (confirm)
- Full Keycloak feature parity.
- Enterprise HA/cluster features in the short term.
- Multi-database support beyond SQLite in the short term.

## Constraints
- Single binary (Rust backend + embedded React UI option)
- SQLite database
- Hexagonal architecture (backend)
- Feature-sliced design (FSD) for UI
- Styling via shadcn + Tailwind

## Current features (high-level)
- Multi-realm
- Flow builder (React Flow)
- Basic RBAC
- gRPC plugin POC
- Basic SSO + OIDC implementations

## Target users
- Self-hosters.
- SaaS teams.
- Internal enterprise use cases.

## Success metrics (confirm)
- Time-to-first-auth: running login flow end-to-end in under 10 minutes.
- Single-binary mode works on a clean machine with minimal config.
- Core flows (OIDC + SSO) work reliably across realms.
- Startup time and memory footprint are measurably small (define targets).

## Notes
- This file is meant to stay concise. Deeper details live in other memory docs.
