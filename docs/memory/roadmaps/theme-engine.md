# Feature Roadmap: Fluid Theme Engine

## Goal
- Build **Fluid**, a native theme engine that makes ReAuth fully customizable without plugins.
- Deliver a high‑performance, single‑binary theming system with minimal external dependencies.
- Provide a best‑in‑class editing UX (drag/drop, live preview, device toggles).

## Principles
- **Decoupled**: backend produces a JSON blueprint; frontend renders with dumb blocks.
- **Performant**: publish‑time compilation, runtime caching, and stable DTOs.
- **Minimal deps**: prefer native browser APIs and in‑house utilities; avoid heavy editor libs.
- **Robust**: strict schema validation and safe fallbacks.

## Functional use cases (must support)
- Brand alignment (exact tokens: colors, fonts, radius).
- Progressive profiling (add fields without code).
- Legal compliance (conditional blocks per locale/country).
- Contextual branding (client_id‑based theme overrides).
- Layout variations (centered, split, minimal, image‑heavy).

## Architecture: Blueprint + Tokens
### Data model (SQLite)
- `theme_tokens`: atomic design values (colors, typography, spacing, radius, shadows).
- `theme_layouts`: shell templates (CenteredCard, SplitScreen, Minimal).
- `theme_nodes`: maps flow nodes to page blueprints (block tree + layout slotting).
- `theme_assets`: images/fonts as BLOBs with metadata + optional cache hint.
- `theme_versions`: draft/published compiled snapshots (immutable JSON).
- `theme_bindings`: realm default + optional client_id overrides.

### Resolution logic (server)
Merge order:
1. Global defaults (embedded base theme).
2. Realm override.
3. Client override (`client_id`, `ui_locales`, or explicit param).

Publish‑time compile to a **Theme Snapshot**:
- Produce a stable JSON blueprint + CSS token map.
- Cache snapshots in memory with ETag/Last‑Modified.

### Rendering contract (frontend)
- `GET /api/realms/{realm}/theme/resolve` returns `{ tokens, layout, blocks, assets }`.
- UI maps `{ block: "Input", props: {...} }` to dumb components.
- No engine‑specific logic leaks into UI; only stable block registry IDs.

## Editor UX (Fluid Builder)
- **Three‑pane workspace**: Blocks + Layers (left), Canvas (center), Inspector (right).
- **Device toggles**: Desktop / Tablet / Mobile previews.
- **Token toggle**: switch inspector between Element Settings and Global Tokens.
- **Top bar actions**: Draft status, Undo/Redo, Preview, Publish.
- **Layout gallery**: choose shells with thumbnail previews.
- **Block library**: Inputs, Buttons, Social, Checkbox, Text, Divider, Legal.

## Now (Phase 2‑A: Foundation)
- Define the **Theme Schema** + JSON validation (versioned).
- Implement storage tables + repositories (`theme_*`).
- Implement **Theme Resolver** with merge + fallback.
- Implement **Theme Snapshot compiler** (publish‑time JSON + CSS tokens).
- Build minimal **Theme Preview** UI (read‑only render from snapshot).
- Add asset ingestion + size limits (store BLOBs in DB; optional filesystem cache).

## Next (Phase 2‑B: Fluid Builder)
- Build the **Fluid Editor** UI (three‑pane, drag/drop, inspector).
- Implement block drag/drop without heavy libs (native DnD + custom hit‑testing).
- Add **Token Panel** with color picker, radius slider, font picker.
- Add **Layout Gallery** and instant canvas updates.
- Implement **Draft vs Published** workflow + audit trail.

## Later (Phase 2‑C: Advanced)
- Per‑client overrides with inheritance UI.
- Accessibility audits + contrast warnings.
- Export/import theme bundles (JSON + assets).
- Theme diffing and rollback history.

## Decisions (best‑practice defaults)
- Assets stored in DB as BLOBs; optional file cache for hot paths.
- Theme binding is per realm by default, with client_id overrides.
- Renderer uses CSS variables generated from tokens to keep DOM light.
- UI blocks are dumb components with strict props validation.

## Risks / dependencies
- Invalid layouts can break auth flows → enforce schema validation + safe defaults.
- Editor performance → virtualize layers list and memoize canvas nodes.
- Asset size growth → enforce per‑asset + per‑theme size limits.

## Open questions
- Do we need per‑locale theme overrides beyond `ui_locales`?
- Should theme snapshots be stored per flow version or per realm only?
