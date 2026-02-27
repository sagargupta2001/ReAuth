# Feature Roadmap: Poly-Plugin Architecture

## Goal
- Build a hybrid plugin system that combines gRPC services and WASM interceptors.

## Current state (code-aligned)
- gRPC plugin POC exists with local process spawning.
- No WASM runtime or UI extensibility layer.

## Now
- Define plugin categories and lifecycles for Backend Logic Plugins (gRPC service type).
- Define plugin categories and lifecycles for Auth Flow and UI Plugins (WASM interceptor type).
- Add WASM runtime (wasmtime or similar) and sandboxed execution.
- Define the Authenticator SPI with `OnAuthenticate` and `OnAction`.
- Implement metadata-driven UI rendering for plugin-defined forms.
- Support plugin static assets at `/plugins/{id}/assets/`.

## Next
- Plugin signing and permission model.
- Health checks, lifecycle hooks, and version compatibility contracts.
- Performance budgets and latency tracing for plugin calls.

## Later
- Plugin registry and discovery tooling.
- Multi-tenant plugin enablement and per-realm policy.

## Risks / dependencies
- WASM sandboxing and capability security are critical.
- UI metadata contracts must be stable for SDKs and Universal Login.
- gRPC and WASM must share a consistent Authenticator trait.

## Open questions
- Preferred WASM runtime and compatibility constraints.
- How to version plugin contracts without breaking tenants.
