# Feature Roadmap: Harbor (Import/Export)

## Goal
- Build a scalable, enterprise-ready import/export system for ReAuth that supports both **atomic portability** (single theme/client/flow) and **system-wide snapshots** (full realm backups/migrations).

## Current state
- Harbor now provides a unified, versioned import/export pipeline with `.reauth` archives, provider-based resource orchestration, schema validation, dry-run support, async jobs, artifact retention, and conflict logging.
- Full realm import/export covers themes, clients, and flows, including theme metadata, assets, and bindings.
- The Harbor Management Hub UI exists with export/import workspaces, live job polling, and a job details sheet for conflicts and export downloads.
- Seeding is now Harbor-backed via bundle import on first boot.
- OIDC client uniqueness is now realm-scoped in SQLite (`UNIQUE (realm_id, client_id)`), which aligns Harbor import/export with cross-realm portability.
- Theme rename imports now use explicit duplicate semantics: `rename` always creates a suffixed theme and records a warning instead of silently reusing an existing matching draft.

## Now
- Keep Harbor stable while the surrounding execution and configuration layers catch up:
  - Harbor backend and UI capabilities are implemented enough to begin integrating them into the rest of the admin product.
  - Remaining effort is mainly contextual UI wiring, richer job UX, and cleanup of the unrelated compile breakages that now sit outside Harbor.

## Next
- Implement system-wide snapshots with selection checklist:
  - Clients, Users, Themes, Flows, RBAC, OIDC clients.
  - Conflict policy: `skip | overwrite | rename`.
- Add contextual UI actions in Themes, Clients, Flows:
  - Export current resource directly from its detail/builder screen.
  - Import replacement bundle from the resource-level workflow where it makes sense.
- Improve Harbor result transparency in the UI:
  - Distinguish created vs updated vs renamed resources in job details.
  - Surface rename warnings and conflict outcomes more prominently in the dashboard.
- Surface async progress more explicitly in Harbor:
  - Progress bars / percentages for active jobs.
  - Better empty/error/loading states around polling.
- Expand job tracking:
  - Add per-item conflict logs + error detail tables for Harbor jobs.
  - Surface conflict metadata in job detail views.
- Add **semantic deduplication** for uniqueness collisions:
  - Lookup existing items (e.g., client_id) and apply conflict policy.
  - If renamed, remap references across related objects (flows, bindings).

## Later
- Optional bundle encryption/signing.
- Schema compatibility policy with **N-2 support**:
  - Manifest `schema_version` + up-converters (`v1_to_v2`, `v2_to_v3`) before import.
- Observability integration (audit events + metrics for imports/exports).
- Cross-realm merge tools and diff previews.
- Integrate Harbor jobs with the Current execution engine (shared task runner + global job view).

## Implementation checklist
- [x] Define `.reauth` bundle layout (`manifest.json`, `data/`, `assets/`) and document the spec.
- [x] Add versioned JSON schemas for bundle types (theme/client/flow/full realm).
- [x] Implement Harbor Resource Provider trait (`export_json`, `import_json`) and registry wiring.
- [x] Build `HarborService` orchestrator with `export_*`, `import_*`, `dry_run_*`.
- [x] Add ExportPolicy with `REDACT` default and `INCLUDE_SECRETS` permission gate.
- [x] Implement transactional dry-run that reports create/update counts without writes.
- [x] Implement asset extraction and ID remapping (themes + references).
- [x] Add conflict policy handling for theme/client/flow imports.
- [x] Use explicit draft metadata for theme conflict checks.
- [x] Add client/flow schema validation (shape checks).
- [x] Add formal bundle-level schema validation.
- [x] Add import summary counts for create/update.
- [x] Add rename handling + basic reference remap for flow/client scope.
- [x] Add cross-resource remap for `client_id` references in flow graphs during full imports.
- [x] Extend full realm import to include themes (new theme creation + bindings).
- [x] Add manifest validation for `exported_at` RFC3339 format and non-empty `source_realm`.
- [x] Remap `client_id` references in additional resources (theme bindings).
- [x] Implement full realm export with selection + theme metadata/bindings.
- [x] Make theme rename semantics explicit: `rename` always creates a suffixed duplicate theme.
- [ ] Add semantic deduplication (lookup, apply conflict policy, remap references).
- [x] Add unified Harbor endpoints (backend) with scope parameters.
- [x] Add client and flow Harbor providers.
- [ ] Add contextual UI actions in Themes/Clients/Flows for export/import.
- [x] Build Harbor Management Hub UI (Export/Import workspaces + Jobs table).
- [x] Wire Export/Import actions to Harbor endpoints (bundle upload, dry_run, conflict_policy).
- [x] Add job tracking table (`harbor_jobs`) + job list endpoints.
- [x] Add import/export progress reporting and failure details.
- [x] Add async job execution for full realm imports (202 + `job_id`).
- [x] Add UI polling strategy for async Harbor jobs.
- [x] Add async export support with persisted bundles + download endpoint.
- [x] Add Harbor job runner abstraction (Current-ready).
- [x] Add async thresholds + `?async=` override parameters.
- [x] Add retention cleanup for Harbor export artifacts.
- [x] Clear artifact metadata when retention deletes files.
- [x] Allow forced async JSON export that returns download link.
- [x] Add conflict logs table + API for job conflicts.
- [x] Add bundle validation and up-converter scaffolding.
- [x] Make exports deterministic (stable ordering + normalized JSON).
- [x] Update seeding to run via Harbor bundle on first boot.
- [x] Add tests for archive I/O and dry-run import.
- [x] Add tests for schema validation, remapping, and conflicts.
- [x] Make OIDC client uniqueness realm-scoped in SQLite for cross-realm Harbor import/export.
- [x] Add regression coverage for importing the same `client_id` into a different realm.
- [x] Add regression coverage for same-realm theme imports with `rename`.

## UI implementation checklist
- [x] Add Harbor nav entry with Lucide icon and page routing.
- [x] Build Harbor Management Hub layout (Export + Import cards, Recent Jobs table).
- [x] Wire Export/Import actions to Harbor API endpoints (archive upload, dry_run, conflict_policy).
- [x] Connect Recent Harbor Jobs table to live data with polling.
- [x] Add job detail view with conflicts and download link.
- [x] Add async export/download state + progress polling.
- [ ] Add contextual export/import actions in Themes, Clients, Flows.

## Risks / dependencies
- Import consistency depends on strict schema validation and correct ID remapping.
- SQLite write contention for large imports; may require chunked transactions.
- Full realm imports must respect realm isolation and avoid cross-resource remap mistakes when rename policy is applied.

## Open questions
- None (resolved):
  - Export secrets are redacted by default; `INCLUDE_SECRETS` requires elevated permission.
  - Compatibility policy: N-2 support using up-converters.
  - Unified backend Harbor API with contextual per-feature UI.
