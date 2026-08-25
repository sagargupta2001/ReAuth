# Spec: Fluid Canvas Drag and Drop

> Distilled from: the follow-up named in `fluid-nested-sections.md` "Out of Scope", 2026-08-25
> Status: Implemented

---

## User Story

As a realm admin composing a theme page, I want to drag blocks directly on the
canvas — where I can see them — instead of only in the sections tree, so that
arranging a layout is a direct manipulation rather than a translation exercise.

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | Drags blocks on the canvas to reorder and reparent them. |
| End User | Sees the resulting layout. No direct interaction. |

---

## Context: what already exists

`fluid-nested-sections.md` shipped the structural primitives and deliberately
deferred canvas drag, requiring that it "reuse the same `moveNode` /
`resolveDrop` primitives". Everything it needs is now in place:

- `resolveDrop` answers "is this drop legal, and where does it land" — cycles,
  depth limit, slot rejection, no-ops — as typed values.
- `moveNode` applies it, owning the index shift.
- One renderer (`renderFluidNode`) wraps every node in exactly one element, so
  drop handlers attach in a single place rather than per node type.

What is missing is only the canvas-side geometry and wiring.

---

## Business Rules

1. Only **authored** nodes are draggable and droppable. The canvas also renders
   the nodes a component expands into (an `Input`'s label, field container, and
   inner input); those are render-time only and must be inert, matching rule 6
   of `fluid-nested-sections.md`.
2. Drop zones follow the **parent's main axis**. In a column the edges are top
   and bottom; in a row they are left and right. Using vertical edges inside a
   row would make the indicator point the wrong way.
3. The middle band of a container nests, exactly as in the tree. A
   non-container has no middle band and splits along its parent's axis.
4. Validity is decided by `resolveDrop`, not re-implemented. A rejected drop
   shows the not-allowed cursor and reports its reason on drop.
5. Dragging a node selects it, so the inspector follows what is being moved.
6. The canvas and the tree share one drag session, so a drag started in either
   can be dropped in the other, and neither can disagree about what a drop means.
7. Drag is only active in inspect mode, which is the canvas's existing "edit the
   structure" mode. Outside it the canvas stays a plain preview.
8. An empty container is a drop target across its whole area — it has no
   children to aim between.

**Edge cases:**

- Dropping a node onto itself, or onto its own descendant, is rejected by
  `resolveDrop` and never reaches the tree.
- A node dragged over one of its own expansion parts resolves to the authored
  ancestor, not the part.
- A zero-sized element (jsdom, or a collapsed node) resolves to the whole-node
  intent rather than guessing an edge.
- The drop indicator never changes layout — it is absolutely positioned, so
  showing it cannot move the thing being aimed at.

---

## Domain Changes

### New Value Objects

```text
DropAxis — which way a parent stacks its children
  - Vertical: edges are top and bottom
  - Horizontal: edges are left and right
```

### Modified Entities

```text
FluidRenderOptions
  + parentDirection?: 'row' | 'column' — set by the Box branch when it recurses,
    so a node knows which way its siblings are laid out
```

No `ThemeNode` change: this is an editing capability over the existing model.

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/**`, `application/**`, `adapters/**` | none — no persistence or API surface |
| `features/fluid/model/dropZones.ts` | **new** — `DROP_EDGE_RATIO`, `DropAxis`, `dropIntentForOffset`, `dropIntentForPoint` (moved out of `sectionTree.ts`, which is tree-specific) |
| `features/fluid/model/sectionTree.ts` | drops the drop-zone geometry it was holding |
| `features/fluid/hooks/useFluidDrag.ts` | renamed from `useSectionDrag`; gains intent-explicit entry points so the canvas can supply its own geometry |
| `features/fluid/lib/renderFluidNode.tsx` | thread `parentDirection` through recursion |
| `features/fluid/components/FluidCanvas.tsx` | drag handlers and the drop indicator in `wrap` |
| `features/fluid/components/FluidBlocksPanel.tsx` | takes the shared drag controller instead of creating its own |
| `pages/theme/builder/FluidBuilderPage.tsx` | owns the one drag session and passes it to both surfaces |

---

## Persistence / API Changes

```text
none — structural edits already go through commitDraft and the draft PUT
```

---

## Flow / Auth Impact

- Flow types affected: none
- Theme or Fluid page impact: builder-only. `FluidLoginScreen` is untouched —
  it drives the same walker but supplies no drag behaviour.

---

## Availability / Admin UX

- Builder behavior:
  - In inspect mode, an authored node on the canvas can be picked up. The
    pointer position within it decides before / after / inside.
  - The active drop shows a line along the parent's axis for a sibling drop, or
    a highlight for nesting. Rejected drops show the not-allowed cursor.
  - Expansion parts never respond to a drag.
- Simple / Advanced mode UX: unchanged

---

## Test Scenarios

1. **Happy path — reorder on the canvas**
   - Given: two text blocks stacked in a column
   - When: the second is dropped on the top edge of the first
   - Then: the order swaps, and the tree shows the same order

2. **Happy path — nest by dropping into a box**
   - Given: a text block and an empty box
   - When: the text is dropped in the middle of the box
   - Then: it becomes a child of the box

3. **Business rule — the axis follows the parent**
   - Given: two blocks inside a row container
   - When: the pointer is on the left half of the second
   - Then: the intent is *before*, not a vertical guess

4. **Business rule — expansion parts are inert**
   - Given: an `Input` rendered on the canvas
   - When: its generated label or field container is dragged, or dropped onto
   - Then: nothing happens; only the authored `Input` participates

5. **Business rule — one shared session**
   - Given: a drag started on a tree row
   - When: it is dropped on the canvas
   - Then: it moves, because both surfaces share one drag controller

6. **Validation failure — cycle**
   - Given: a box containing a child
   - When: the box is dropped inside its own child
   - Then: rejected with a reason, and the tree is unchanged

7. **Business rule — drag is inspect-mode only**
   - Given: inspect mode is off
   - When: a canvas node is dragged
   - Then: nothing is draggable

8. **Undo**
   - Given: a completed canvas move
   - When: undo is triggered
   - Then: the previous tree is restored, identically to a tree-driven move

---

## Out of Scope

- Free/absolute positioning. Dropping still resolves to a place in the tree.
- Multi-select drag.
- Auto-scrolling the canvas while dragging near its edge.
- Dragging a new block from the picker onto the canvas — the picker inserts by
  location today, and that is a separate affordance.
- Resize handles on canvas nodes.

---

## Resolved Questions

- [x] **Drag is inspect-mode only.** Outside it the canvas is a preview and
      clicks belong to the rendered controls. The canvas also takes the drag
      controller as an *optional* prop, so a surface that passes none — any
      read-only preview — is inert by construction rather than by a flag.
- [ ] Should the canvas auto-scroll when dragging near an edge? Still deferred;
      independent of this slice and only matters for pages taller than the
      viewport.

## Notes from implementation

- The drag session moved up to `FluidBuilderPage`. Two independent hooks both
  mutating the same tree would have been precisely the duplication Phase 3 spent
  its time removing, and lifting it is what makes tree ↔ canvas drag work at all.
- Auto-expanding a collapsed container on hover moved out of the hook and into
  the sections panel, driven off `dropTarget`. Collapse is a tree-only view
  concern and had no business in a drag session the canvas also uses.
- `dropIntentForOffset` and `DROP_EDGE_RATIO` moved from `model/sectionTree.ts`
  to `model/dropZones.ts`. They were never tree-specific; the canvas needed the
  same geometry with a different axis.
