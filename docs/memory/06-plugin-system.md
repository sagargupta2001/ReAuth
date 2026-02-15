# Plugin System

This is a gRPC-based plugin POC managed by a local `PluginManager`.

## Plugin discovery and lifecycle
- Manager scans `plugins/` for subfolders containing `plugin.json`.
- Plugins are enabled/disabled via API endpoints.
- When enabled, the manager spawns the plugin process and performs a gRPC handshake.
- Active plugins are tracked in memory; status is derived from disk manifests + active registry.

## Plugin directory
- Base directory is `plugins/` (relative to repo in dev).
- In production, the plugins directory is a sibling of the executable.
- Determination logic: `reauth/crates/reauth_core/src/bootstrap/plugins.rs`

## Manifest format (`plugin.json`)
See schema in `reauth/crates/plugin/manager/src/plugin.rs`.
Key fields:
- `id`, `name`, `version`
- `executable` with OS-specific paths (`linux_amd64`, `windows_amd64`)
- `frontend` config (`entry`, `route`, `sidebarLabel`)
- `events.subscribes_to` (list of event types)

## Handshake protocol (current)
- Plugin prints a single line on stdout with 5 delimited fields.
- Delimiter: `|`
- Protocol type must be `grpc`.
- Manager reads the line, builds a channel, and calls `Handshake.GetPluginInfo` to verify.
- Timeout is controlled by `Settings.plugins.handshake_timeout_secs`.

Proto definitions:
- `reauth/proto/plugin.v1.proto`

## API endpoints (current)
Routes under `/api/plugins`:
- `GET /api/plugins/manifests` -> list plugin statuses
- `POST /api/plugins/{id}/enable`
- `POST /api/plugins/{id}/disable`
- `GET /api/plugins/{id}/say-hello` (Greeter example)

Static plugin assets are served from `/plugins/*` (non-API path).

## Event delivery to plugins
- Domain events are broadcast to active plugins via gRPC.
- `PluginEventGateway` checks manifest subscriptions and calls `EventListener.OnEvent`.
- Event payloads are JSON strings derived from `DomainEvent`.

Relevant code:
- Manager: `reauth/crates/plugin/manager/src/manager.rs`
- Event gateway: `reauth/crates/reauth_core/src/adapters/plugin_gateway/event_gateway.rs`
- Plugin API handlers: `reauth/crates/reauth_core/src/adapters/web/plugin_handler.rs`

## Current limitations
- POC only: one example Greeter RPC is wired.
- No sandboxing or permission model enforced yet.
- No signature verification or trusted plugin registry.
