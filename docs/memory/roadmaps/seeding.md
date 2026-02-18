# Feature Roadmap: Database Seeding

## Goal
- Make default data seeding modular, idempotent, and scalable as new defaults are added.

## Current state (code-aligned)
- [x] Seeding lives in a monolithic `bootstrap/seed.rs` function.
- [x] Defaults include realm, flows, admin user/role, and default OIDC client.
- [x] Seeding is run on every startup (after migrations).
- [ ] No seed versioning or history tracking.

## Now
1. **Modularize seeders**
   - Split `seed.rs` into feature‑scoped modules (`realm`, `flows`, `admin`, `oidc`, `rbac`, …).
   - Introduce a simple `SeedContext` to share common deps.
2. **Idempotency contract**
   - Ensure each seeder is safe to run multiple times without duplicating data.

## Next
1. **Seed history + versioning**
   - Add a `seed_history` table: `name`, `version`, `checksum`, `applied_at`.
   - Only re-run seeders if their version/checksum changes.
2. **Seeder registry**
   - Add a `Seeder` trait and a registry that executes seeders in order.
   - Allow per‑seeder transactions.

## Later
- Externalize default data into `config/seed/*.toml` (or embedded JSON).
- Add a `--seed-only` flag for CI/dev.
- Add tests for idempotency and data integrity.

## Risks / dependencies
- Schema changes require synchronizing seeder versions.
- Default OIDC client auto‑sync must remain compatible with seed history.

## Open questions
- Should seeders be re-run automatically on version bump, or only with a flag?
