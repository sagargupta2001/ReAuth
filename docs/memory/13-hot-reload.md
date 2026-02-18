# Hot Reload (Config)

## Purpose
Enable safe, dev-friendly configuration updates without restarting the server. The watcher reloads settings when the active config file changes and applies the new values to runtime reads.

## How it works
- **Watch target** is resolved once at startup via `Settings::resolve_config_watch_path()`:
  1. `REAUTH_CONFIG` (if set and exists)
  2. `reauth.toml` next to the executable (if present)
  3. `config/default.toml` (dev fallback)
- A `notify` watcher listens for changes to that file (non-recursive).
- On change events, updates are **debounced** (200ms) to avoid partial writes.
- The new config is loaded via `Settings::new()` and replaces the in-memory settings inside `Arc<RwLock<Settings>>`.

Relevant code:
- `reauth/crates/reauth_core/src/bootstrap/initialize.rs` (watcher + reload loop)
- `reauth/crates/reauth_core/src/config.rs` (`resolve_config_watch_path`)
- `reauth/crates/reauth_core/src/bootstrap/app_state.rs` (shared `settings` storage)

## What reloads immediately
Anything read through `state.settings.read().await` at request time, including:
- `ui.dev_url` (dev UI proxy)
- `cors.allowed_origins`
- `default_oidc_client.web_origins` (CORS allowlist)

## What requires restart
Some settings are only read at startup and **won’t fully apply** until restart:
- `server.scheme`, `server.host`, `server.port` (bind address)
- `database.url`, `database.data_dir` (DB connection)
- `auth.jwt_secret`, `auth.jwt_key_id`, `auth.issuer` (token signing/validation)

When these change, the server logs a warning after reload.

## Safety notes
- Hot reload is **best effort**: if the config fails validation, the reload is skipped and a warning is logged.
- The watcher is disabled when no config file exists.
- This is intended for **dev/local workflows**. In production, prefer explicit restarts for core settings.

## Operational tips
- For quick iteration, keep a `reauth.toml` beside the binary and edit it during runtime.
- Use `--check-config` before a reload to validate changes.
- Use `--print-config` to verify effective values (secrets redacted).
