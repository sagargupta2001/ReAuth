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
- Implement `HarborService` in `/src/application` as an orchestrator with a **Resource Provider Registry**:
  - Each domain registers a provider that implements a Harbor trait (e.g., `ThemeProvider`, `ClientProvider`, `FlowProvider`).
  - Providers implement `export_json()` and `import_json()` and are wired at startup.
- Add a transaction-based import pipeline with:
  - Pre-validate JSON + schema.
  - Optional dry-run using a temporary transaction.
  - Asset extraction + ID remapping (theme assets and node references).
- Start with atomic portability: theme, client, flow.
- Add contextual UI export/import actions in Themes, Clients, Flows.
- Implement **ExportPolicy**:
  - Default `REDACT` secrets in exports (`"${REDACTED}"` placeholder).
  - `INCLUDE_SECRETS` gated by a “Full Backup” permission.

## Next
- Implement system-wide snapshots with selection checklist:
  - Clients, Users, Themes, Flows, RBAC, OIDC clients.
  - Conflict policy: `skip | overwrite | rename`.
- Add Import/Export Management Hub under Realm Settings:
  - Two-column layout: Export Workspace + Import Workspace.
  - Export checklist + “Include Secrets” toggle + “Generate .reauth Bundle”.
  - Import drag-and-drop + manifest preview + conflict policy + dry-run toggle.
  - Recent Harbor Jobs table (status, items processed, date).
- Add import/export job tracking:
  - `import_jobs`, `export_jobs`, `import_conflicts`, `import_logs` tables.
  - Progress and failure reporting for long-running imports.
- Unify seeding:
  - `seed.rs` delegates to Harbor using a local bundle (e.g., `config/seed/default-theme.reauth`).
  - First-run bootstrapping via `system-init.json` (full realm export).
- Add **semantic deduplication** for uniqueness collisions:
  - Lookup existing items (e.g., client_id) and apply conflict policy.
  - If renamed, remap references across related objects (flows, bindings).

## Later
- Deterministic export ordering for Git-friendly diffs.
- Optional bundle encryption/signing.
- Schema compatibility policy with **N-2 support**:
  - Manifest `schema_version` + up-converters (`v1_to_v2`, `v2_to_v3`) before import.
- Observability integration (audit events + metrics for imports/exports).
- Cross-realm merge tools and diff previews.

## Implementation checklist
- [ ] Define `.reauth` bundle layout (`manifest.json`, `data/`, `assets/`) and document the spec.
- [ ] Add versioned JSON schemas for bundle types (theme/client/flow/full realm).
- [ ] Implement Harbor Resource Provider trait (`export_json`, `import_json`) and registry wiring.
- [ ] Build `HarborService` orchestrator with `export_*`, `import_*`, `dry_run_*`.
- [ ] Add ExportPolicy with `REDACT` default and `INCLUDE_SECRETS` permission gate.
- [ ] Implement transactional dry-run that reports create/update counts without writes.
- [ ] Implement asset extraction and ID remapping (themes + references).
- [ ] Add semantic deduplication (lookup, apply conflict policy, remap references).
- [ ] Add unified Harbor endpoints (backend) with scope parameters.
- [ ] Add contextual UI actions in Themes/Clients/Flows for export/import.
- [ ] Build Harbor Management Hub UI (Export/Import workspaces + Jobs table).
- [ ] Add job tracking tables (`import_jobs`, `export_jobs`, `import_conflicts`, `import_logs`).
- [ ] Add import/export progress reporting and failure details.
- [ ] Add up-converters for N-2 schema compatibility.
- [ ] Make exports deterministic (stable ordering + normalized JSON).
- [ ] Update seeding to run via Harbor bundle on first boot.
- [ ] Add tests for schema validation, remapping, conflicts, and dry-run.

## Risks / dependencies
- Import consistency depends on strict schema validation and correct ID remapping.
- SQLite write contention for large imports; may require chunked transactions.
- Full realm imports must respect realm isolation and avoid client_id/global uniqueness collisions (mitigated via semantic deduplication + remapping).

## Open questions
- None (resolved):
  - Export secrets are redacted by default; `INCLUDE_SECRETS` requires elevated permission.
  - Compatibility policy: N-2 support using up-converters.
  - Unified backend Harbor API with contextual per-feature UI.
