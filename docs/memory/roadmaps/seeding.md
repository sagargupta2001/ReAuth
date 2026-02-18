# Feature Roadmap: Database Seeding

## Goal
- Make default data seeding modular, idempotent, and scalable as new defaults are added.

## Current state (code-aligned)
- [x] Seeding is orchestrated in `bootstrap/seed.rs` with feature‑scoped modules.
- [x] Defaults include realm, flows, admin user/role, and default OIDC client.
- [x] Seeding is run on every startup (after migrations).
- [x] Seed versioning and history tracking are in place.
- [x] Seed logic split into feature‑scoped modules with shared context.
- [x] Seed history table tracks `name` + `version` + `checksum`.
- [x] Seeder registry executes versioned seeders in order.

## Now
1. **Modularize seeders**
   - Keep modules aligned as new defaults are added.
2. **Idempotency contract**
   - Ensure each seeder is safe to run multiple times without duplicating data.

## Next
1. **Seeder transactions**
   - Allow per‑seeder transactions for atomic seeding steps.
2. **Admin seeding refinement**
   - Improve role lookup if `super_admin` already exists (avoid silent skip).

## Later
- Externalize default data into `config/seed/*.toml` (or embedded JSON).
- Add a `--seed-only` flag for CI/dev.
- Add tests for idempotency and data integrity.

## Risks / dependencies
- Schema changes require synchronizing seeder versions.
- Default OIDC client auto‑sync must remain compatible with seed history.

## Open questions
- Should seeders be re-run automatically on version bump, or only with a flag?
