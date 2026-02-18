# Feature Roadmap: Configuration & Environment

## Goal
- Make environment configuration explicit, portable, and easy to override for local/dev/prod.

## Current state (code-aligned)
- [x] Embedded defaults load from `config/default.toml` with optional local override.
- [x] Optional external config: `reauth.toml` next to the executable or `--config` / `REAUTH_CONFIG`.
- [x] `.env` is loaded before config resolution.
- [x] Env overrides use `REAUTH__` prefix with `__` separators.
- [x] `server.public_url` drives derived defaults (OIDC issuer + default redirect URIs).
- [x] Default OIDC client is auto-synced from config (managed-by-config).
- [x] Config precedence documented in README/DevOps docs.
- [x] Vite proxy target configurable via `VITE_API_PROXY_TARGET`.
- [x] Startup validation with clear errors for invalid config.
- [x] Startup diagnostics log (public URL, data dir, DB URL, UI dev URL, CORS count).
- [x] CORS allowlist configurable via `cors.allowed_origins`.
- [x] `--print-config` supported for resolved config output.
- [x] Build generates a commented `reauth.toml` template beside the binary (if missing).
- [x] `--init-config` supported to create a local config template on demand.

## Now
1. **Config introspection**
   - Add a `--print-config`/`--init-config` docs section with examples (README).
2. **Env list parsing**
   - Define supported syntax for list env vars (comma‑separated vs JSON) and document it.

## Later
- Support additional config formats (e.g., YAML) if needed.
- Hot reload for dev config (watch the config file and reload on change without restarting the server).

## Open questions
- Env var naming convention: **`REAUTH__` with `__` separators** (accepted).
