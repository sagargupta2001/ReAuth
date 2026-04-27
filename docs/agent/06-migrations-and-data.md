# Migrations And Data Guide

Use this guide whenever a change touches persisted state.

## Core stance

- ReAuth is SQLite-first.
- Migrations are forward-only.
- Schema changes must support the single-binary self-hosted model.
- Avoid designs that require external data infrastructure.

## Migration rules

- Add a new SQL migration file in `migrations/`.
- Follow the existing timestamp-based naming convention:
  - `YYYYMMDDHHMMSS_description.sql`
- Prefer additive changes:
  - new tables
  - new nullable columns
  - new indexes
  - compatible backfills
- Be careful with destructive schema changes. If needed, design a staged migration path.

## Implementation checklist for persistence changes

1. Add the migration.
2. Update domain models if persisted shape changed.
3. Update repository ports if new access patterns are needed.
4. Update SQLite adapters.
5. Update seed logic if defaults or bootstrap behavior changed.
6. Add or update repository/integration tests.
7. Update relevant docs/specs.

## Data modeling guidance

- Keep realm scoping explicit in persisted records when the data is realm-bound.
- Prefer simple, auditable schema over abstract generic storage.
- Use JSON columns only where the product already intentionally models flexible payloads, such as flow graphs, script payloads, or theme blueprints.
- Do not use a migration as a substitute for a missing domain model decision. Decide the model first, then encode it.

## Backfills and compatibility

- Assume existing local installations may already have real data.
- New fields should have safe defaults or be nullable until the code is fully migrated.
- If a feature needs a data transition, describe it in the feature spec.
