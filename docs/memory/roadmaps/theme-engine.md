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
- `GET /api/realms/{realm}/theme/resolve` returns `{ tokens, layout, nodes, assets }`.
- UI renders **primitives** (`Box`, `Text`, `Image`, `Icon`) and **components** (`Input`, etc).
- Components are resolved via a stable registry and expand to primitives + slots at render/compile time.
- No engine‑specific logic leaks into UI; only stable node/component registry IDs.
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
  "nodes": [
    {
      "id": "n1",
      "type": "Box",
      "props": {},
      "layout": { "direction": "column", "gap": 12, "align": "stretch", "padding": [16, 16, 16, 16] },
      "size": { "width": "fill", "height": "hug" },
      "children": [
        {
          "id": "n2",
          "type": "Text",
          "props": { "text": "Sign in" },
          "size": { "width": "hug", "height": "hug" }
        },
        {
          "id": "n3",
          "type": "Component",
          "component": "Input",
          "props": { "name": "email", "label": "Email" },
          "slots": {
            "prefix": {
              "id": "n4",
              "type": "Icon",
              "props": { "name": "mail" }
            }
          }
        }
      ]
    }
  ],
  "assets": [
    { "id": "uuid", "filename": "hero.png", "mime_type": "image/png", "url": "/api/realms/{realm}/theme/{theme_id}/assets/{asset_id}" }
  ]
}
```

Schema sketch (nodes + slots)
```json
{
  "$id": "theme.snapshot.node",
  "type": "object",
  "required": ["id", "type"],
  "properties": {
    "id": { "type": "string" },
    "type": { "enum": ["Box", "Text", "Image", "Icon", "Component"] },
    "component": { "type": "string" },
    "props": { "type": "object" },
    "layout": {
      "type": "object",
      "properties": {
        "direction": { "enum": ["row", "column"] },
        "gap": { "type": "number" },
        "align": { "enum": ["start", "center", "end", "stretch"] },
        "padding": {
          "type": "array",
          "items": { "type": "number" },
          "minItems": 4,
          "maxItems": 4
        }
      }
    },
    "size": {
      "type": "object",
      "properties": {
        "width": { "enum": ["fixed", "hug", "fill"] },
        "height": { "enum": ["fixed", "hug", "fill"] },
        "width_value": { "type": "number" },
        "height_value": { "type": "number" }
      }
    },
    "children": { "type": "array", "items": { "$ref": "theme.snapshot.node" } },
    "slots": { "type": "object", "additionalProperties": { "$ref": "theme.snapshot.node" } }
  },
  "allOf": [
    { "if": { "properties": { "type": { "const": "Component" } } }, "then": { "required": ["component"] } }
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
- **Header**: Page selector dropdown + draft/publish actions.
- **Floating action bar**: Undo / Redo / Inspect toggle.
- **Top bar actions**: Draft status, Publish, Save.
- **Layout gallery**: choose shells with thumbnail previews.
- **Block library**: Inputs, Buttons, Social, Checkbox, Text, Divider, Legal.
- **Block layover**: omni-style picker with preview panel.

## Current focus (Phase 2‑D: Box Model + Componentization)
- Move from flat blocks to a **Box Model**: containers control layout, primitives render content.
- Introduce **Nested Blocks / Slots** so complex blocks are composed from primitives.
- Ship **system Components** (starting with `Input`) that expose only curated properties.
- Add **Auto‑Layout** controls to the inspector (direction, gap, alignment, padding).
- Add **Sizing** controls (Fixed / Hug / Fill) aligned with Figma semantics.

## Block Model v2 (Box Model)
- **Atomic primitives**: `Box`, `Text`, `Image`, `Icon`.
- **Containers** define layout (flex direction, alignment, gap, padding).
- **Components** are templates of primitives + slots with an exposed prop surface.
- **Nested trees** are first‑class; complex blocks expand into sub‑trees at render/compile time.

## Migration notes (legacy flat blocks)
Legacy blocks will be wrapped into the Box Model during draft load or publish.
1. `Text` block becomes a `Text` primitive node with `size` set to `hug`.
2. `Input` block becomes a `Component` node with `component = "Input"` and legacy props mapped to component props.
3. `Button` block becomes a `Component` node with `component = "Button"` and legacy `variant` mapped to component props.
4. `Link` block becomes a `Component` node with `component = "Link"` and legacy `href/target` preserved.
5. `Divider` block becomes a `Component` node with `component = "Divider"`.
6. `Image` block becomes an `Image` primitive node with `asset_id/alt` props retained.
7. Blocks with spacing props are wrapped in a parent `Box` that applies `padding` and `gap`.
8. Blocks using `slot = "brand"` remain tagged via `props.slot` until slots are promoted to first‑class container slots.
9. Layout-only wrappers (if any) become `Box` nodes with `children` preserved.

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

## Phase 3: Figma-scale block growth (proposed)

Four refactors, ordered. Each is justified by something measured in the current
code, not by aesthetics. The inventory they act on is generated into
`docs/memory/22-fluid-capability-matrix.md`.

### R1 — One renderer, two hosts (biggest lever)

`FluidCanvas.renderNode` is 312 lines and `FluidLoginScreen.renderNode` is 352.
**233 of those lines are byte-identical** once whitespace is trimmed — roughly
70% duplication. Every new block and every render-level styling prop is written
twice, and the divergence has shipped bugs at least three times: the
`ProviderButtons` canvas gap, the two `resolveVisibleFlag` implementations that
disagreed on `visible: 0`, and the five renderer defaults in
`18-fluid-theme-builder.md` §5.4.3.

The parts that genuinely differ are only four, and they form a clean seam:

1. **Wrapping** — the canvas adds a selection ring and a click target; the
   runtime adds nothing.
2. **Visibility** — the runtime gates on `visible` / `visible_if`; the builder
   always renders, so the node stays selectable.
3. **Leaf interactivity** — the runtime wires `FormField`, `Input`,
   `PasswordInput`, actions, OAuth, and passkeys; the builder renders inert
   equivalents.
4. **Unresolved data** — the builder shows the binding itself (`{message}`,
   dimmed); the runtime resolves it against auth context. `ProviderButtons` is
   the same split: placeholder versus live providers.

Extract `renderFluidNode(node, host)` into `features/fluid/lib/`, with a
`FluidHost` interface covering exactly those four concerns. Both components
become thin adapters over one tree walker.

- Payoff: a new block is one branch, not two. Renderer parity stops being a test
  and becomes structural — §4 of the capability matrix becomes trivially true.
- Guarded by: `FluidCanvasStyling`, `FluidCanvasLoginPage`, and
  `FluidRendererParity` already assert the behaviour that must not change.

### R2 — A colour control for the inspector (cheap, independent of R1)

`ThemeColorControl` makes design-token-vs-literal an explicit choice and previews
the token's real colour, but it is used **only** by the theme settings panel.
Every colour in the block inspector is a raw text input — `props.color` in
Typography, and `props.background` / `props.border_color` in the Box Appearance
section.

Add `InspectorFieldKind.Color` (a kind, a descriptor, a case in
`InspectorField.tsx`) and switch those fields to it.

- Payoff: block-level colours can reference tokens, so a palette change
  propagates instead of being pinned per block. Directly serves the "brand
  alignment" use case above.

### R3 — A component registry instead of a dispatch chain (after R1) — **done, scoped down**

Proposed as "one registry entry per block, colocating definition, preview,
inspector section, and render". Two thirds of that did not survive contact with
the code and were deliberately dropped:

- **Preview stays in `BLOCK_PREVIEWS`.** It is already a
  `Record<FluidBlockId, …>`, so a missing preview is a compile error and the bug
  it would prevent cannot occur. Merging it into `FLUID_BLOCKS` would only move
  JSX into `model/`, which is where the per-item *data* lives.
- **Inspector sections stay section-oriented.** They are cross-cutting —
  Typography applies to Text, Button, and Link — so a per-block layout would
  either duplicate them or reinvent `appliesTo` under another name. Declaring
  applicability on the section is already the right model.

What shipped is the part with a real payoff: the component `if` chain in the
walker became `COMPONENT_RENDERERS`, a map keyed by lowercased component name.
Adding a component block is an entry, and — the reason it is worth doing — the
set is now **enumerable**, so the capability matrix imports `RENDERED_COMPONENTS`
instead of pattern-matching the renderer's source for branches.

### R4 — Grouped style objects instead of a flat prop bag (largest; own spec)

The `Input` component alone carries **nine** prefixed styling props —
`field_background`, `field_border_color`, `field_border_width`, `field_padding`,
`field_radius`, `label_color`, `label_size`, `label_spacing`, `label_weight` —
each re-declaring a generic concept because slot styling has no structure. Three
more composed components at that rate is ~30 props that all mean "background" or
"radius".

Figma's model is per-node style groups: fills, strokes, effects, corner radius,
typography. Add `node.style` groups any node can carry, and address a slot as
`node.slots.label.style` rather than inventing `label_*` props. A styling
capability is then added once and applies to every block.

Shipped as `docs/specs/fluid-style-groups.md`. `node.style` carries `fill`,
`stroke`, `corners`, `spacing`, and `typography`, plus `parts` for a composed
component's expansion parts. All nine prefixed props are gone from the schema.

Compatibility is a read-time mapping, not a migration: `resolveNodeStyle` prefers
the group and falls back to the legacy prop, so stored blueprints render
unchanged forever. `extractNodesFromBlueprint` normalizes the draft on load, so a
page converges on the new shape the first time it is saved. Nothing rewrites
stored rows.

Two things changed from the draft on contact with the code, both recorded in the
spec: component parts are `style.parts.<part>` rather than authored slots (the
label and field container are render-time nodes and must stay out of the authored
tree), and `spacing.padding` is a single number — the four-number tuple is a
container's inner padding and stays in `layout`.

### Explicitly not now

- An external or plugin block SDK — there is no consumer.
- Server-side rendering or code-generated blueprints — contradicts the decoupled
  principle above.
- A CSS-in-JS or style-system dependency — minimal deps stands; Tailwind plus
  tokens is working.
- ~~Canvas drag and drop~~ — shipped after R1, and it did reuse `resolveDrop` /
  `moveNode` as predicted. R1 is what made it small: drop handlers attach in the
  walker's single `wrap`, not once per node type.

Sequence: **R1 → R2 (independent, can run in parallel) → R3 → R4**.

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
- [x] Add Box primitive with flex layout props (direction, gap, alignment, padding).
- [x] Add sizing model for blocks (Fixed / Hug / Fill) and persist in draft snapshots.
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
- [x] Add Box primitive with flex layout props (direction, gap, alignment, padding).
- [x] Add sizing model for blocks (Fixed / Hug / Fill) and persist in draft snapshots.
- [x] Update schema + validator to support nested blocks and named slots.
- [x] Introduce Component definitions with exposed props (system + future custom).
- [x] Convert `Input` to a system Component with internal tree (Label Text, FieldContainer Box, PrefixIcon Icon, ActualInput Primitive, ErrorHint Text).
- [x] Expose Label typography + padding-bottom via Input component props.
- [x] Expose FieldContainer border/background/padding via Input component props.
- [x] Add PrefixIcon slot to the Input component.
- [x] Add ActualInput primitive node inside Input component.
- [x] Add ErrorHint text with conditional visibility in the Input component.
- [x] Add inspector Auto‑Layout panel (direction, gap, alignment, padding).
- [x] Update renderer to expand Components into primitives + containers at render/compile time.
- [x] Update tree view to show component parts or expose named slots for editing.
- [x] Move Undo/Redo/Inspect controls into the floating action bar.
- [x] Refresh theme preview queries after save/publish/activate/rollback/draft-create.
- [ ] Externalize default theme tokens/layout/page blueprints into JSON seed assets.
- [ ] Allow default theme seed to be sourced from an exported Fluid theme bundle (configurable path/env).
- [x] Implement theme bundle import/export (JSON + assets) for the Fluid editor.
- [x] Add per‑client override editor (inheritance + preview).
- [x] Add basic contrast warnings in the inspector (text vs theme background).
- [x] Add theme version snapshot viewer in history (JSON payload).
- [x] Add theme diffing + rollback UI in theme history (snapshot diff vs active).
- [x] Add Flow Builder ↔ Fluid template binding UI (node → page selector).
- [x] Persist flow node → page bindings in flow config.
- [x] Validate flow bindings on theme switch and show warnings + fallback behavior.
- [x] Nested section editing: drag into containers, indent/outdent, depth limit, empty-container targets (`fluid-nested-sections.md`).
- [x] Generate the capability matrix (blocks, styling options, tokens, gaps) from the schemas.
- [x] Expose Box surface props (background, border colour/width, corner radius) in the inspector.
- [x] R1: collapse the two `renderNode` switches into one host-driven tree walker (`lib/renderFluidNode.tsx` + `FluidShell`).
- [x] R2: add an inspector colour control with design-token references.
- [x] R3: make component rendering a registry (`COMPONENT_RENDERERS`) the matrix can enumerate.
- [x] R4: grouped style objects on nodes (`lib/nodeStyle.ts`), replacing the nine prefixed slot-styling props.
- [x] Separate palette entries from render targets (`RENDER_TARGETS`), so presets can share a node kind.
- [x] Add Checkbox (consent / remember-me), Columns, and Heading blocks.
- [x] Canvas drag and drop, sharing one drag session with the sections tree (`docs/specs/fluid-canvas-drag.md`).
- [x] Drag a new block out of the picker onto either surface.
- [x] Retire the `Tab` keyboard trap: indent/outdent moved to `Alt+Right` / `Alt+Left`.
- [x] Converge seeded blueprints on style groups, and assert it in the seed audit.
- [x] Add Radio Group, Select, and Legal Text blocks.

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
