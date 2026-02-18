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

## Now
1. **Document config precedence**
   - Document order: embedded defaults → optional local defaults → `reauth.toml`/`--config` → env.
   - Provide examples for overriding port, issuer, and default client URLs.
2. **UI dev proxy**
   - Make Vite proxy target configurable via `VITE_API_PROXY_TARGET`.

## Later
- Support config files (e.g., TOML/YAML) in addition to env vars.
- Hot reload for dev config.

## Open questions
- Which env var naming convention should be standardized (`REAUTH_*`)?
