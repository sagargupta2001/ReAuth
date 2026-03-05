# Feature Roadmap: Harbor (Import/Export)

## Goal
- Build a scalable, enterprise-ready import/export system for ReAuth that supports both **atomic portability** (single theme/client/flow) and **system-wide snapshots** (full realm backups/migrations).

## Current state
- Theme import/export exists at a feature level (`/api/realms/:realm/themes/:theme_id/export` and `/import`), but there is no unified, versioned bundle format or shared import pipeline.
- Seeding is code-driven (`/src/bootstrap/seed.rs`) and not yet powered by a reusable import/export service.

## Now
- Define the Harbor bundle format and manifest:
  - `.reauth` bundle (zip or tar.gz).
  - `manifest.json` with `version`, `exported_at`, `source_realm`, `type`.
  - `data/` (JSON) and `assets/` (binary) split.
- Add schema validation for bundles (versioned JSON schema).
- Implement `HarborService` in `/src/application` with `export_*`, `import_*`, and `dry_run_*`.
- Add a transaction-based import pipeline with:
  - Pre-validate JSON + schema.
  - Optional dry-run using a temporary transaction.
  - Asset extraction + ID remapping (theme assets and node references).
- Start with atomic portability: theme, client, flow.
- Add contextual UI export/import actions in Themes, Clients, Flows.

## Next
- Implement system-wide snapshots with selection checklist:
  - Clients, Users, Themes, Flows, RBAC, OIDC clients.
  - Conflict policy: `skip | overwrite | rename`.
- Add Import/Export Management Hub under Realm Settings.
- Add import/export job tracking:
  - `import_jobs`, `export_jobs`, `import_conflicts`, `import_logs` tables.
  - Progress and failure reporting for long-running imports.
- Unify seeding:
  - `seed.rs` delegates to Harbor using a local bundle (e.g., `config/seed/default-theme.reauth`).
  - First-run bootstrapping via `system-init.json` (full realm export).

## Later
- Deterministic export ordering for Git-friendly diffs.
- Optional bundle encryption/signing.
- Schema compatibility policy (forward/backward version checks).
- Observability integration (audit events + metrics for imports/exports).
- Cross-realm merge tools and diff previews.

## Risks / dependencies
- Import consistency depends on strict schema validation and correct ID remapping.
- SQLite write contention for large imports; may require chunked transactions.
- Full realm imports must respect realm isolation and avoid client_id/global uniqueness collisions.

## Open questions
- Should bundle exports redact secrets (client secrets, keys) by default?
- What is the compatibility policy for old bundle versions?
- Do we want a unified “Harbor” API surface or per-feature endpoints with a shared backend pipeline?
