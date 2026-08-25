# Spec: Fluid Style Groups

> Distilled from: Phase 3 R4 of `docs/memory/roadmaps/theme-engine.md`, 2026-08-25
> Status: Implemented

---

## User Story

As a realm admin styling a theme page, I want the same styling controls —
background, border, radius, spacing, typography — on every block and on every
part of a composed block, so that learning one control teaches me all of them
and a new block arrives already stylable.

As a ReAuth developer, I want a styling capability to be added once rather than
once per block, so that the control surface can grow to Figma scale without the
prop list growing quadratically.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Styles blocks and component slots in the Fluid inspector. |
| End User | Sees the styled auth page. No direct interaction. |
| Operator | Owns existing blueprints that must keep rendering unchanged. |

---

## Context: the problem being solved

`node.props` is a flat, untyped bag, so every styling capability is declared
per block *and* per slot. The `Input` component alone carries nine prefixed
props that re-declare generic concepts:

```text
field_background   field_border_color   field_border_width
field_padding      field_radius
label_color        label_size           label_spacing        label_weight
```

`Box` separately declares `background`, `border_color`, `border_width`,
`radius`. These are the same four ideas written twice, with different names,
read by different code paths. A third composed component would add nine more.

The generated inventory in `docs/memory/22-fluid-capability-matrix.md` shows
the shape of it: §2 lists `Label` and `Field` sections that exist only because
an `Input`'s parts have no addressable style of their own.

Two things already landed that this depends on:

- **One renderer** (`lib/renderFluidNode.tsx`) — style resolution now has a
  single consumer instead of two divergent ones.
- **A generated capability matrix** — the before/after of this change is
  measurable rather than asserted.

---

## Business Rules

1. Every node may carry a `style` object. The same groups are available on every
   node type; a group a renderer cannot apply to that type is ignored, not an
   error.
2. Style groups are `fill`, `stroke`, `corners`, `spacing`, and `typography`.
   Each is an object, so a group can grow a property without touching any block.
3. A composed component's parts are addressed as `node.style.parts.<part>`, not
   as prefixed props on the parent. `label_color` becomes
   `style.parts.label.typography.color`.
4. `props.*` styling keys remain readable forever. Existing blueprints are
   persisted JSON and are never rewritten in place.
5. On load, legacy props are normalized into style groups by a pure mapping. The
   normalized tree is what the renderer and inspector see.
6. When both a legacy prop and its style-group equivalent are present, the style
   group wins. A blueprint can only reach that state by hand-editing.
7. Saving a draft writes style groups. Legacy keys that were normalized are not
   written back, so a page converges on the new shape the first time it is saved.
8. A colour in any group may hold a design-token reference or a literal, the same
   choice the inspector already offers.
9. The inspector's style sections are declared once and target `style.<group>.<key>`,
   replacing the per-block `Appearance`, `Label`, and `Field` sections.

**Edge cases:**

- A blueprint with neither legacy props nor style groups renders on renderer
  defaults, exactly as today.
- A hand-edited blueprint with an unknown group key keeps the key and ignores it.
  Round-tripping must not silently delete authored data.
- A node whose slot was removed loses that slot's style with it; no orphan
  entries are kept.
- Normalization runs on the *draft*, so undo/redo sees normalized trees only —
  a legacy tree is never an undo target mid-session.
- Theme export/import carries whatever shape the draft holds. An export taken
  before this ships imports and normalizes on load.

---

## Domain Changes

### New Value Objects

```text
NodeStyle — the styling a node carries, all groups optional
  - fill?: FillStyle
  - stroke?: StrokeStyle
  - corners?: CornerStyle
  - spacing?: SpacingStyle
  - typography?: TypographyStyle

FillStyle
  - color?: string — literal or design-token reference

StrokeStyle
  - color?: string
  - width?: number

CornerStyle
  - radius?: number | string

SpacingStyle
  - padding?: number — space *around* the block
  - margin_top?: number
  - margin_bottom?: number

TypographyStyle
  - size?: string
  - weight?: string
  - color?: string
  - align?: 'left' | 'center' | 'right'

NodeStyle.parts?: Record<string, NodeStyle>
  Styling for the nodes a composed component expands into, keyed by part name.
```

### Modified Entities

```text
ThemeNode
  + style?: NodeStyle — grouped styling, additive to `props`
```

`props` is unchanged and stays the home for *content* (`text`, `label`, `href`,
`name`, `placeholder`) and behaviour (`visible`, `visible_if`, `slot`). Only
styling moves.

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/theme.rs` | **`ThemeNodeInstance` gains `style` plus a `#[serde(flatten)] extra`** — it is a *typed* view of blueprint JSON, not opaque, so unnamed keys were being dropped |
| `domain/theme_pages.rs` | seeded blueprints may adopt `style`; not required to ship |
| `application/theme_service.rs` | none in behaviour; gains round-trip tests for the resolve path |
| `adapters/web/...` | none |
| `adapters/persistence/...` | none |
| `entities/theme/model/types.ts` | add `NodeStyle` and `ThemeNode.style` |
| `features/fluid/lib/nodeStyle.ts` | **new** — `normalizeNodeStyle`, `readStyle`, `withStyle` |
| `features/fluid/lib/nodeVisuals.ts` | read style groups first, legacy props as fallback |
| `features/fluid/lib/renderFluidNode.tsx` | consume resolved style instead of raw props |
| `features/fluid/lib/componentRegistry.ts` | expand parts from `style.parts`, dropping `field_*` / `label_*` |
| `features/fluid/lib/nodeUtils.ts` | normalize on `extractNodesFromBlueprint` |
| `features/fluid/model/inspectorFields.ts` | add `FieldTarget.Style` with a group + key |
| `features/fluid/model/inspectorSchema.ts` | replace `Appearance`/`Label`/`Field` with style sections |
| `features/fluid/model/capabilityMatrix.test.ts` | report style groups alongside props |

---

## Persistence Changes

### New Migration(s)

```text
none — `style` round-trips inside blueprint_json like `children` did
```

### Data Notes

- No backfill and no rewrite of stored rows. Normalization is a read-time
  mapping in the UI; rule 7 makes the write path converge lazily.
- `theme_pages.rs::seed_audit_tests` must accept either shape while seeds are
  mixed, then be tightened once seeds adopt `style`.
- Snapshot diffing categorises by path prefix, so `nodes[].style.*` lands under
  `nodes` without change.

---

## API Changes

### New HTTP Endpoints

```text
none
```

### Modified Endpoints (if any)

```text
none — the draft PUT already carries the whole blueprint tree
```

---

## Flow / Auth Impact

- Flow types affected: none
- New nodes: none
- Existing nodes modified: none
- Async pause/resume impact: none
- Theme or Fluid page impact: every page re-renders through the normalized
  style path. Both renderers share `renderFluidNode`, so parity is structural —
  but the normalization itself must be covered by tests at the `nodeVisuals`
  level, where the legacy fallback lives.

---

## Availability / Admin UX

- System/operator prerequisites: none
- Realm policy: none
- Flow composition: none
- Builder behavior:
  - The inspector shows the same style sections for every block, with groups a
    node cannot use hidden rather than shown-and-inert.
  - A slot selected in the sections tree gets the same style sections as any
    other node, which is what removes the `Label` and `Field` panels.
  - No migration prompt. A page converts silently on first save.
- Simple mode UX: unchanged
- Advanced mode UX: unchanged

---

## Test Scenarios

1. **Happy path — style a block**
   - Given: a `Box` with no styling
   - When: a fill colour and corner radius are set in the inspector
   - Then: the node carries `style.fill.color` and `style.corners.radius`, and
     both renderers paint them

2. **Happy path — style a component part**
   - Given: an `Input` is selected
   - When: the Label Color is set
   - Then: it writes `style.parts.label.typography.color`, and no `label_color`
     prop is created

3. **Validation failure — unknown group is preserved**
   - Given: a hand-edited blueprint with `style.glow = {}`
   - When: the page is loaded and saved
   - Then: the unknown group survives the round trip and is ignored at render

4. **Business rule edge case — legacy props still render**
   - Given: a stored blueprint using `background`, `border_color`, `field_radius`
   - When: it is rendered without being saved
   - Then: it looks byte-identical to how it looked before this change

5. **Business rule edge case — style wins over a legacy prop**
   - Given: a node with both `props.background` and `style.fill.color`
   - When: rendered
   - Then: `style.fill.color` is used

6. **Business rule edge case — lazy convergence**
   - Given: a legacy blueprint opened in the builder
   - When: the draft is saved with no edits
   - Then: the saved tree carries style groups and no normalized legacy keys

7. **Renderer parity**
   - Given: a tree using every style group
   - When: rendered by `FluidCanvas` and by `FluidLoginScreen`
   - Then: both produce the same structure and applied styles

8. **Error handling — malformed style value**
   - Given: `style.stroke.width = "thick"`
   - When: rendered
   - Then: the property is dropped rather than emitted as invalid CSS, matching
     how `resolveCssLength` already guards lengths

9. **Capability matrix**
   - Given: the style groups have shipped
   - When: the matrix is regenerated
   - Then: the nine `field_*` / `label_*` props are gone and the inspector's
     style sections apply to every block

---

## Out of Scope

- Shared/named styles reusable across nodes (Figma's "styles"). This slice gives
  every node its own style; reuse is a later concept built on top.
- Effects beyond corner radius — shadows, blurs, opacity layers.
- Multiple fills or strokes per node. One of each, matching what the renderers
  can express today.
- Responsive or breakpoint-scoped style values.
- Rewriting stored blueprints server-side, or any migration.
- Adopting `style` in the seeded blueprints in `theme_pages.rs`. Additive and
  independent once the read path lands.

---

## Resolved Questions

- [x] **`layout` stays separate.** It describes how a node arranges its
      *children* — direction, gap, align, justify, and the inner padding tuple —
      not how the node paints. Folding it in would have collided with
      `spacing.padding`, which is the space *around* a block.
- [x] **`size` stays separate.** Geometry with a typed home already.
- [x] **The whole page converts on load, and Save persists it.**
      `extractNodesFromBlueprint` normalizes, so the builder, the inspector, and
      undo/redo only ever see the new shape.
- [x] **The runtime needs no normalization pass.** It only ever *reads*, and
      `resolveNodeStyle` resolves legacy props transparently, so
      `FluidLoginScreen` renders either shape without converting anything. This
      turned out simpler than the spec anticipated.

## Post-implementation defect: the backend was not opaque

This spec asserted that blueprints are "opaque JSON to the service" and that
`domain/theme.rs` needed no change. **That was wrong, and it shipped a bug.**

`ThemeNodeInstance` is a *typed* struct. `parse_blueprint` deserialises into it
and the resolve path re-serialises from it, so every key the struct does not
name is silently dropped. `style` was such a key.

The failure was invisible in the obvious place and visible everywhere else:

| Path | Shape | Result |
|---|---|---|
| Draft read/save | raw `serde_json::Value` | style preserved — **builder looked correct** |
| Publish → version snapshot | raw `serde_json::Value` | style preserved in storage |
| Resolve → preview and runtime | `ThemeNodeInstance` | **style dropped** |

So a colour or alignment set in the builder persisted fine, and then vanished on
the preview screen and the real login page.

Fixed by naming `style` on the struct and adding `#[serde(flatten)] extra` so the
*next* unnamed key survives instead of being deleted. No data was lost: storage
always held the full JSON, so the fix restored existing themes with no repair.

Guarded by `blueprint_round_trip_tests` in `theme_service.rs`, which asserts
style survives on nodes, children, slots, and component parts — and that an
unknown key does too. Four of its five tests fail without the fix.

**The lesson worth keeping:** "the backend treats this as opaque" is a claim to
verify by reading the struct, not to assume from the fact that a field is JSON.

## Deviations from the spec as drafted

Two things changed once the code was in front of me, both recorded above:

1. **Parts, not slots.** The draft said a slot's style is
   `node.slots.<key>.style`. But an `Input`'s label and field container are
   *not* authored slots — they are generated by `componentRegistry` at render
   time and, per rule 6 of `fluid-nested-sections.md`, never appear in the
   authored tree. Addressing them as slots would have meant promoting them into
   the authored tree, changing the sections tree and the seed audit for no gain.
   `style.parts` keeps the authored tree untouched and still removes all nine
   prefixed props.
2. **`SpacingStyle.padding` is a single number**, matching what `props.padding`
   always was — the space around a block. The draft's four-number tuple was the
   *container's* inner padding, which is `layout.padding` and stays there.
