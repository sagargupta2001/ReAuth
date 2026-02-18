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
- [x] Env list parsing for comma‑separated values (documented).
- [x] `--check-config` supported to validate config and exit.
- [x] Minimal `--help` output lists supported flags.
- [x] Warn when `server.public_url` origin doesn’t match the bind origin.
- [x] README includes `--print-config` / `--init-config` examples.
- [x] Hot reload for config file changes (with non‑reloadable fields warning).
- [x] `--check-config` includes public URL/bind origin mismatch warning.
- [x] Hot reload warns when the watched config file disappears.
- [x] README CLI examples expanded as flags grew.

## Now
1. **Config guardrails**
   - Warn if the config watch target disappears (done; keep watching for regressions).
2. **Config UX**
   - Keep CLI examples up to date as new flags are added.

## Later
- Support additional config formats (e.g., YAML) if needed.

## Open questions
- Env var naming convention: **`REAUTH__` with `__` separators** (accepted).
