# Roadmap

This is the cross-cutting roadmap. Feature-specific roadmaps live in `reauth/docs/memory/roadmaps/`.

## Now (next 2-4 weeks)
- Implement flow version checksum in `FlowManager` (`TODO_HASH` currently).
- Decide on `auth_sessions.execution_state` + `last_ui_output`: wire into runtime or remove from schema.
- Capture IP/user-agent in execution flow (TODO in `execution_handler.rs`).
- Align realm schema with flow bindings used in code (`client_authentication_flow_id`, `docker_authentication_flow_id`) or remove those bindings.
- Decide how to handle `code_challenge_method` (currently only SHA-256 is implemented).

## Next (1-3 months)
- Implement flow simulation (UI button exists, no backend implementation yet).
- Expand node library and config schemas (e.g., registration/reset-specific nodes).
- Solidify plugin security model (signing, permissions, isolation).
- Define performance targets (startup time, memory) and add benchmarks.

## Later (3+ months)
- Audit log persistence and export (currently event bus + log broadcast).
- Multi-tenant operational tooling (backup/restore, data migration utilities).

## Risks and dependencies
- Flow type bindings for `client` and `docker` are referenced in code but missing in schema.
- Plugin system security is currently minimal (POC) and may block enterprise usage.
- OIDC `client_id` is globally unique in schema; may need per-realm uniqueness.
