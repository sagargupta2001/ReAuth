# ADR: Fluid Box Model + Componentization

Status: Accepted
Date: 2026-03-03

## Context
The current Fluid theme engine uses a flat "Block" model where complex UI elements (e.g., inputs) are opaque blocks. This limits layout control, makes inspector controls coarse, and prevents Figma-like composition where components are built from smaller primitives and arranged via auto-layout.

We want to enable nested layout, precise spacing, and reusable components while keeping the renderer simple and the schema stable.

## Decision
Adopt a Box Model architecture for Fluid:
- Introduce atomic primitives: `Box`, `Text`, `Image`, `Icon`.
- Make containers (`Box`) the primary layout mechanism with flex direction, alignment, gap, and padding.
- Promote **components** to first-class nodes with named slots; components expand into primitives + containers at render/compile time.
- Define system components starting with `Input`, composed of:
  - Label `Text`
  - FieldContainer `Box`
  - PrefixIcon `Icon`
  - ActualInput `Primitive`
  - ErrorHint `Text`
- Add sizing semantics (Fixed / Hug / Fill) to align with Figma-like behavior.

## Alternatives considered
- Add more fields to the existing flat Block model (e.g., a `margin` prop on `Input`).
  - Rejected: increases complexity without enabling nested composition.
- Keep a single “Layout” block and treat all others as leaves.
  - Rejected: blocks still remain opaque, limiting inspector control and component reuse.

## Consequences
- Theme snapshot schema must support nested nodes, named slots, and sizing.
- Renderer must expand components to primitives at render/compile time.
- Fluid inspector and tree view must support container layout controls and component slots.
- Existing theme drafts require a migration path to wrap or map legacy blocks into Box/Component nodes.
