# Feature Roadmap: Configuration & Environment

## Goal
- Make environment configuration explicit, portable, and easy to override for local/dev/prod.

## Current state (code-aligned)
- [ ] Server port is hardcoded.
- [ ] Environment configuration is not centralized.

## MVP scope (prioritized)
1. **Server config**
   - Introduce configuration layer (env + defaults) for port, host, DB path, and feature toggles.
   - Provide clear docs for env var usage and defaults.
2. **Validation**
   - Validate required env vars on boot (fail fast with clear errors).

## Later
- Support config files (e.g., TOML/YAML) in addition to env vars.
- Hot reload for dev config.

## Open questions
- Which env var naming convention should be standardized (`REAUTH_*`)?
