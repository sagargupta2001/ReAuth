# Spec: Fluid Style Groups

> Distilled from: Phase 3 R4 of `docs/memory/roadmaps/theme-engine.md`, 2026-08-25
> Status: Draft

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
3. A slot's style is addressed as `node.slots.<key>.style`, not as a prefixed
   prop on the parent. `label_color` becomes `slots.label.style.typography.color`.
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
  - padding?: [number, number, number, number]
  - margin_top?: number
  - margin_bottom?: number

TypographyStyle
  - size?: string
  - weight?: string
  - color?: string
  - align?: 'left' | 'center' | 'right'
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
| `domain/theme.rs` | none — `style` rides inside the opaque blueprint JSON |
| `domain/theme_pages.rs` | seeded blueprints may adopt `style`; not required to ship |
| `application/theme_service.rs` | none — blueprints stay opaque |
| `adapters/web/...` | none |
| `adapters/persistence/...` | none |
| `entities/theme/model/types.ts` | add `NodeStyle` and `ThemeNode.style` |
| `features/fluid/lib/nodeStyle.ts` | **new** — `normalizeNodeStyle`, `readStyle`, `withStyle` |
| `features/fluid/lib/nodeVisuals.ts` | read style groups first, legacy props as fallback |
| `features/fluid/lib/renderFluidNode.tsx` | consume resolved style instead of raw props |
| `features/fluid/lib/componentRegistry.ts` | expand slots with their own style, dropping `field_*` / `label_*` |
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

2. **Happy path — style a slot**
   - Given: an `Input` whose label slot is selected
   - When: a typography colour is set
   - Then: it writes `slots.label.style.typography.color`, and no `label_color`
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

## Open Questions

- [ ] Should `layout` (direction, gap, align, justify, padding) fold into
      `style.spacing`, or stay a separate container concern? Keeping it separate
      is proposed: it describes how a node arranges *children*, not how it paints.
- [ ] Does `size` (fixed/hug/fill) become a style group too? Proposed no, for the
      same reason — it is geometry, and it already has a typed home.
- [ ] Should the write path convert on save (rule 7) or only when a node is
      edited? Converting the whole page on save is simpler and makes the
      capability matrix's gap analysis converge faster.
- [ ] Is one normalization pass at `extractNodesFromBlueprint` enough, or does
      the runtime snapshot path need its own? The runtime reads
      `snapshot.nodes` directly and does not currently call that helper.
