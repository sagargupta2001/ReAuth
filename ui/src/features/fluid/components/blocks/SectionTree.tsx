import type { ThemeNode } from '@/entities/theme/model/types'
import { AddBlockButton } from '@/features/fluid/components/blocks/AddBlockButton'
import { SectionTreeNode } from '@/features/fluid/components/blocks/SectionTreeNode'
import {
  SECTION_TREE_NODE_DEPTH,
  SECTION_TREE_SCAFFOLD,
  type SectionScaffoldRow,
} from '@/features/fluid/model/sectionTree'
import { cn } from '@/lib/utils'

interface SectionTreeProps {
  nodes: ThemeNode[]
}

/** Page > shell > blocks hierarchy for the currently edited theme page. */
export function SectionTree({ nodes }: SectionTreeProps) {
  return (
    <div className="space-y-3 text-xs">
      {SECTION_TREE_SCAFFOLD.map((row) => (
        <ScaffoldRow key={row.key} row={row} insertIndex={nodes.length} />
      ))}
      <div className="space-y-1">
        {nodes.length === 0 && (
          <p className="text-muted-foreground pl-8 text-[11px]">
            Add blocks to build this page.
          </p>
        )}
        {nodes.map((node, index) => (
          <SectionTreeNode
            key={node.id}
            node={node}
            depth={SECTION_TREE_NODE_DEPTH}
            index={index}
            isRoot
          />
        ))}
      </div>
    </div>
  )
}

function ScaffoldRow({
  row,
  insertIndex,
}: {
  row: SectionScaffoldRow
  insertIndex: number
}) {
  return (
    <div
      className={cn(
        'group text-foreground/80 flex items-center justify-between rounded-md px-2 py-1 text-[11px] font-semibold',
        row.depth === 0 ? 'pl-2' : 'pl-6',
      )}
    >
      <span>{row.label}</span>
      <AddBlockButton
        anchorKey={row.key}
        insertIndex={insertIndex}
        label={`Add block to ${row.label}`}
        className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
      />
    </div>
  )
}
