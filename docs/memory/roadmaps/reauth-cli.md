# Feature Roadmap: reauth-cli

## Goal
- Provide developer tooling that makes plugins and integration effortless.

## Current state
- No CLI tooling.

## Now
- `reauth-cli plugin init <name>` for gRPC plugin scaffolds.
- `reauth-cli plugin init <name> --wasm` for WASM plugin scaffolds.
- `reauth-cli plugin dev` for hot-reload plugin development.
- `reauth-cli plugin build` for packaging and validation.

## Next
- Template registry and versioned plugin templates.
- Validate plugin manifests and runtime compatibility.
- `reauth-cli plugin test` for local contract tests.

## Later
- Publish and install workflow for shared plugins.
- Integrated telemetry and performance profiling for plugin calls.

## Risks / dependencies
- Depends on a stable plugin contract and manifest schema.
- Requires WASM and gRPC runtime conventions to be finalized.

## Open questions
- Distribution channel for templates and plugins.
- How to manage plugin version compatibility across releases.
