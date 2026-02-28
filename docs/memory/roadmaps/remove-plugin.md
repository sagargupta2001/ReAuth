# Roadmap: Remove Plugin System

## Goal
- Remove the external plugin system entirely and keep ReAuth as a single-binary product.
- Shift extensibility to internal, native features (theme engine + embedded scripting) without sidecars.

## Current state
- gRPC plugin POC exists with manager, gateway, UI surfaces, and example plugin assets.
- Event routing includes plugin targets alongside HTTP webhooks.
- Workspace includes plugin crates and proto definitions.

## Step-by-step plan (ordered)
1. Inventory all plugin surfaces and dependencies across backend, UI, migrations, config, and docs.
2. Remove plugin UI surfaces, routes, sidebar entries, and Omni Search items related to plugins.
3. Remove plugin API endpoints and routing in the backend (handlers, routes, and auth gates).
4. Remove the plugin runtime layer (manager, gateway, bootstrap initialization, and any event dispatch logic).
5. Remove gRPC plugin proto definitions, generated code, and gRPC/plugin dependencies from Cargo.
6. Remove plugin data model artifacts (tables/columns, migrations, seed data, and any config settings).
7. Delete plugin assets and examples (`plugins/hello-world`) plus static asset serving paths.
8. Delete plugin crates (`crates/plugin/manager`, `crates/plugin/sdk`) and update workspace membership.
9. Simplify the workspace to a single crate if feasible (move `crates/reauth_core` to `reauth/src`, update paths).
10. Update documentation to remove plugin references and add the new internal extensibility direction.
11. Run verification (build, tests, UI navigation smoke) and confirm no plugin references remain.

## Risks / dependencies
- Requires careful cleanup of gRPC/proto build steps and any generated artifacts.
- Event routing logic and UI need adjustments once plugin targets are removed.

## Open questions
- When should the theme engine + embedded scripting roadmap be formalized and added to memory docs?
