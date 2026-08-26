/**
 * Custom drag payload for the sections tree. The transferred value is the
 * dragged node's **id** — indices shift the moment a sibling moves, and a path
 * captured at drag start is stale by the time the drop fires.
 */
export const SECTION_DRAG_MIME_TYPE = 'application/reauth-fluid-node'

/**
 * Drag payload for a *new* block coming out of the picker: a catalog id rather
 * than a node id, because nothing has been created yet.
 */
export const BLOCK_DRAG_MIME_TYPE = 'application/reauth-fluid-block'

/** Left indent applied per tree depth, in pixels. */
export const SECTION_TREE_INDENT_PX = 12

/** Depth at which the page's root nodes are rendered. */
export const SECTION_TREE_NODE_DEPTH = 2

/**
 * Maximum number of levels in the authored tree; root nodes are level 1.
 *
 * Six covers realistic auth pages while keeping the recursive renderers and the
 * tree readable. It constrains new operations only — a blueprint loaded from an
 * older version, or hand-edited, renders as authored however deep it is.
 */
export const MAX_NESTING_DEPTH = 6

/**
 * Non-editable rows shown above the page's nodes. They exist to communicate the
 * page > shell > blocks hierarchy and to offer an "add block" affordance.
 */
export interface SectionScaffoldRow {
  key: string
  label: string
  depth: number
}

export const SECTION_TREE_SCAFFOLD: readonly SectionScaffoldRow[] = [
  { key: 'page', label: 'Page', depth: 0 },
  { key: 'layout', label: 'Layout Container', depth: 1 },
]

/** Anchor key for the picker opened from the panel header. */
export const HEADER_ANCHOR_KEY = 'header'

/** Anchor key for the picker opened from a node row. */
export function nodeAnchorKey(nodeId: string): string {
  return `node-${nodeId}`
}

/** Anchor key for the picker that adds a block *inside* a container row. */
export function insideAnchorKey(nodeId: string): string {
  return `inside-${nodeId}`
}
