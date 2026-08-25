import type { NodeDropIntent } from '@/features/fluid/lib/nodeUtils'

/**
 * Where a pointer position inside an element means a block should land.
 *
 * Shared by the sections tree and the canvas so the two cannot disagree about
 * what a drop means. The tree is always a vertical stack of rows; the canvas
 * follows whatever axis the node's parent lays its children out on.
 */

/**
 * Fraction of the element, at each end, that means "drop as a sibling".
 * The remaining middle band nests, on elements that accept children.
 */
export const DROP_EDGE_RATIO = 0.25

/** Which way a parent stacks its children, and therefore where its edges are. */
export const DropAxis = {
  Vertical: 'vertical',
  Horizontal: 'horizontal',
} as const

export type DropAxis = (typeof DropAxis)[keyof typeof DropAxis]

export function axisForDirection(direction: 'row' | 'column' | undefined): DropAxis {
  return direction === 'row' ? DropAxis.Horizontal : DropAxis.Vertical
}

/**
 * Which drop a pointer position along one axis means.
 *
 * An element that cannot contain children has no middle band at all, so the two
 * halves split cleanly into before/after. An unmeasurable element — no extent,
 * or a pointer position the environment did not report — resolves to the
 * whole-element intent rather than pretending the pointer is at a particular
 * edge. jsdom reports every element as zero-sized, so that case is not
 * hypothetical.
 */
export function dropIntentForOffset(
  offset: number,
  extent: number,
  acceptsChildren: boolean,
): NodeDropIntent {
  if (!Number.isFinite(extent) || extent <= 0 || !Number.isFinite(offset)) {
    return acceptsChildren ? 'inside' : 'after'
  }
  if (!acceptsChildren) {
    return offset < extent / 2 ? 'before' : 'after'
  }
  if (offset < extent * DROP_EDGE_RATIO) return 'before'
  if (offset > extent * (1 - DROP_EDGE_RATIO)) return 'after'
  return 'inside'
}

export interface DropPoint {
  clientX: number
  clientY: number
}

export interface DropRect {
  left: number
  top: number
  width: number
  height: number
}

/**
 * Which drop a pointer position over a rendered element means.
 *
 * The axis is the *parent's*, not the element's: a block inside a row is
 * dropped to the left or right of its siblings, and reading its vertical edges
 * would point the indicator the wrong way.
 */
export function dropIntentForPoint(
  point: DropPoint,
  rect: DropRect,
  options: { acceptsChildren: boolean; axis: DropAxis },
): NodeDropIntent {
  const horizontal = options.axis === DropAxis.Horizontal
  return dropIntentForOffset(
    horizontal ? point.clientX - rect.left : point.clientY - rect.top,
    horizontal ? rect.width : rect.height,
    options.acceptsChildren,
  )
}
