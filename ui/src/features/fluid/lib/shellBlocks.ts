/**
 * Splits a page's root nodes across the shell's panes.
 *
 * `props.slot` is the only placement signal a blueprint carries: `brand` sends
 * a node to the `SplitScreen` brand pane, anything else (or nothing) is form
 * content. Both renderers partitioned nodes with their own inline filters,
 * which is three copies of the same three predicates.
 */
export interface ShellPanes<T> {
  brand: T[]
  form: T[]
  /** Every non-brand block, for the shells that have only one pane. */
  nonSplit: T[]
}

export function partitionShellBlocks<T extends { props?: Record<string, unknown> }>(
  blocks: T[],
): ShellPanes<T> {
  const slotOf = (node: T) => {
    const props = node.props ?? {}
    return String(props.slot ?? 'form')
  }
  return {
    brand: blocks.filter((node) => Boolean(node.props) && slotOf(node) === 'brand'),
    form: blocks.filter((node) => !node.props || slotOf(node) === 'form'),
    nonSplit: blocks.filter((node) => !node.props || slotOf(node) !== 'brand'),
  }
}
