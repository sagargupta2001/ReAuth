/** Custom drag payload used for reordering root-level section nodes. */
export const SECTION_REORDER_MIME_TYPE = 'application/reauth-fluid-reorder'

/** Left indent applied per tree depth, in pixels. */
export const SECTION_TREE_INDENT_PX = 12

/** Depth at which the page's root nodes are rendered. */
export const SECTION_TREE_NODE_DEPTH = 2

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
