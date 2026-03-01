# Feature Roadmap: Fluid Theme Engine

## Goal
- Build **Fluid**, a native theme engine that makes ReAuth fully customizable without plugins.
- Deliver a high‑performance, single‑binary theming system with minimal external dependencies.
- Provide a best‑in‑class editing UX (drag/drop, live preview, device toggles).
- Ensure Fluid pages can be mapped to auth flow nodes (template ↔ node binding).

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
- `themes`: theme metadata per realm.
- `theme_tokens`: atomic design values (colors, typography, spacing, radius, shadows).
- `theme_layouts`: shell templates (CenteredCard, SplitScreen, Minimal).
- `theme_nodes`: maps **page keys** to page blueprints (block tree + layout slotting).
- `theme_assets`: images/fonts as BLOBs with metadata + optional cache hint.
- `theme_versions`: draft/published compiled snapshots (immutable JSON).
- `theme_bindings`: realm default + optional client_id overrides.
- `system_pages`: in‑code registry of default pages + templates (login, register, forgot, etc).

### Resolution logic (server)
Merge order:
1. Global defaults (embedded base theme).
2. Realm override.
3. Client override (`client_id`, `ui_locales`, or explicit param).

Publish‑time compile to a **Theme Snapshot**:
- Produce a stable JSON blueprint + CSS token map.
- Cache snapshots in memory with ETag/Last‑Modified.

Page resolution:
1. If theme has an override for the page → use it.
2. Else → use the system default page template.

### Rendering contract (frontend)
- `GET /api/realms/{realm}/theme/resolve` returns `{ tokens, layout, blocks, assets }`.
- UI maps `{ block: "Input", props: {...} }` to dumb components.
- No engine‑specific logic leaks into UI; only stable block registry IDs.
- Runtime login page renders from the resolved snapshot (not hardcoded UI).

Theme Snapshot schema (draft):
```json
{
  "theme_id": "uuid",
  "version_id": "uuid",
  "tokens": {
    "colors": { "primary": "#1C64F2", "background": "#FFFFFF" },
    "typography": { "font_family": "Inter", "base_size": 16 }
  },
  "layout": { "shell": "CenteredCard", "slots": ["main", "aside"] },
  "blocks": [
    { "block": "Text", "props": { "text": "Sign in" }, "children": [] },
    { "block": "Input", "props": { "name": "email", "label": "Email" }, "children": [] }
  ],
  "assets": [
    { "id": "uuid", "filename": "hero.png", "mime_type": "image/png", "url": "/api/realms/{realm}/theme/{theme_id}/assets/{asset_id}" }
  ]
}
```

## Editor UX (Fluid Builder)
- **Triple‑sidebar workspace**:
  - Primary sidebar (collapsed): Sections + Theme Settings.
  - Secondary sidebar (expanded): Tree view of the page or Tokens.
  - Center canvas: live rendering with inspect mode.
  - Right sidebar: contextual inspector.
- **Device toggles**: Desktop / Tablet / Mobile previews.
- **Header**: Page selector dropdown + icon-only Undo/Redo + Inspect toggle.
- **Top bar actions**: Draft status, Publish, Save.
- **Layout gallery**: choose shells with thumbnail previews.
- **Block library**: Inputs, Buttons, Social, Checkbox, Text, Divider, Legal.
- **Block layover**: omni-style picker with preview panel.

## Now (Phase 2‑A: Foundation)
- Define the **Theme Schema** + JSON validation (versioned).
- Implement storage tables + repositories (`theme_*`).
- Implement **Theme Resolver** with merge + fallback.
- Implement **Theme Snapshot compiler** (publish‑time JSON + CSS tokens).
- Build minimal **Theme Preview** UI (read‑only render from snapshot).
- Add asset ingestion + size limits (store BLOBs in DB; optional filesystem cache).

## Next (Phase 2‑B: Fluid Builder)
- Build the **Fluid Editor** UI (triple‑sidebar, drag/drop, inspector).
- Implement block drag/drop without heavy libs (native DnD + custom hit‑testing).
- Add **Token Panel** with color picker, radius slider, font picker (in secondary sidebar).
- Add **Layout Gallery** and instant canvas updates.
- Implement **Draft vs Published** workflow + audit trail.
- Replace version UUIDs in UI with semantic aliases (v1/v2/v3).

## Later (Phase 2‑C: Advanced)
- Per‑client overrides with inheritance UI.
- Accessibility audits + contrast warnings.
- Export/import theme bundles (JSON + assets).
- Theme diffing and rollback history.

## Implementation checklist
- [x] Create core storage tables (`themes`, `theme_tokens`, `theme_layouts`, `theme_nodes`, `theme_assets`, `theme_versions`, `theme_bindings`).
- [x] Add `is_system` flag to themes + seed a non-deletable ReAuth Default theme.
- [x] Add system page registry (login/register/forgot/etc) + default templates.
- [x] Implement theme repository + resolver service.
- [x] Add theme list/detail/version API endpoints.
- [x] Add `/themes/pages` endpoint to serve system page templates.
- [x] Add theme admin UI (sidebar, details, history, settings).
- [x] Add Fluid shell (three‑pane editor + full‑screen layout).
- [x] Add theme publish endpoint + UI action.
- [x] Add draft fetch/save endpoints and UI wiring for Fluid.
- [x] Persist token edits in Fluid (save draft + publish).
- [x] Persist layout selection in Fluid (layout gallery → draft).
- [x] Persist block tree edits in Fluid (drag/drop → draft).
- [x] Add asset ingestion API + UI upload panel.
- [x] Add preview endpoint for draft snapshots (no publish).
- [x] Add block property editing (selected block → inspector).
- [x] Add draft preview button in Fluid (open preview endpoint).
- [x] Render blocks on the canvas preview (not just list).
- [x] Render blocks in SplitScreen brand pane (brand slot blocks).
- [x] Add per-block style controls (spacing + alignment) in inspector.
- [x] Add block-level typography controls (font size/weight/color).
- [x] Add slot-aware previews for non-SplitScreen layouts (brand slot ignored).
- [x] Add per-block width/size controls (button/input/image).
- [x] Add asset selection for image blocks (use uploaded assets).
- [x] Add block reordering in the Fluid tree view.
- [x] Add Fluid page selector + per-page overrides (default vs customized).
- [x] Page-aware preview/resolve fallback (missing override → system template).
- [x] Ensure default theme auto-created for newly created realms.
- [x] Add “Reset page to default” action in Fluid (removes override).
- [x] Triple‑sidebar layout (primary + secondary + right inspector).
- [x] Header redesign (page selector dropdown, icon-only undo/redo, inspect toggle).
- [x] Secondary sidebar tree view (page → layout → blocks).
- [x] Block layover picker (40/60 preview panel).
- [x] Replace version UUIDs with semantic aliases in UI (map to UUIDs).
- [x] Remove canvas drag/drop add (use block picker + tree).
- [x] Anchor block picker to clicked `+` button.
- [x] Add undo/redo history in Fluid.
- [x] Add input type control for input blocks (text/email/password).
- [x] Render login page from the active theme snapshot (Fluid runtime renderer).
- [x] Pass `client_id` to theme resolution for contextual branding on login.
- [x] Allow custom pages (`custom.*`) in theme drafts + list them in Fluid.
- [x] Add “Create new page” action in Fluid page selector.

## Upcoming integration (Flow Builder ↔ Fluid)
- Add a **Template Selector** per Flow Node (bind node → page key).
- Persist node → page bindings in flow config.
- Validate flow bindings on theme switch (warning if active flow uses missing template).
- Provide fallback behavior (auto-fallback to system page + warning).

## Decisions (best‑practice defaults)
- Assets stored in DB as BLOBs; optional file cache for hot paths.
- Theme binding is per realm by default, with client_id overrides.
- Renderer uses CSS variables generated from tokens to keep DOM light.
- UI blocks are dumb components with strict props validation.

## Risks / dependencies
- Invalid layouts can break auth flows → enforce schema validation + safe defaults.
- Editor performance → virtualize layers list and memoize canvas nodes.
- Asset size growth → enforce per‑asset + per‑theme size limits.
- System page changes → ensure existing themes gracefully inherit new pages.
- Flow ↔ Theme mapping adds a new consistency constraint across realms/themes.

## Open questions
- Do we need per‑locale theme overrides beyond `ui_locales`?
- Should theme snapshots be stored per flow version or per realm only?
- Should custom pages be realm‑level (shared across themes) or theme‑local?
