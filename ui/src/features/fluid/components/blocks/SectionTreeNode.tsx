import { AlertCircle, GripVertical, Trash2 } from 'lucide-react'

import { Button } from '@/components/button'
import type { ThemeNode } from '@/entities/theme/model/types'
import { AddBlockButton } from '@/features/fluid/components/blocks/AddBlockButton'
import { useSectionsPanel } from '@/features/fluid/components/blocks/sectionsPanelContext'
import { labelForNode } from '@/features/fluid/model/blockCatalog'
import {
  SECTION_TREE_INDENT_PX,
  nodeAnchorKey,
} from '@/features/fluid/model/sectionTree'
import { cn } from '@/lib/utils'

const ROW_ACTION_CLASS = 'h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100'

interface SectionTreeNodeProps {
  node: ThemeNode
  depth: number
  /** Position among the page's root nodes. Only root nodes can be reordered. */
  index?: number
  isRoot?: boolean
}

/** One row of the section tree, plus its slots and children. */
export function SectionTreeNode({ node, depth, index, isRoot = false }: SectionTreeNodeProps) {
  const { selectedNodeId, errorsByNodeId, onSelectNode, onRemoveNode, reorder } =
    useSectionsPanel()

  const label = labelForNode(node)
  const isSelected = selectedNodeId === node.id
  const errorCount = errorsByNodeId.get(node.id)?.length ?? 0
  const hasError = errorCount > 0
  const dragIndex = isRoot && typeof index === 'number' ? index : null
  const slots = Object.entries(node.slots ?? {})
  const children = node.children ?? []

  return (
    <div className="space-y-1">
      <div
        className={cn(
          'group flex items-center gap-2 rounded-md py-1 pr-2 transition-colors',
          isSelected
            ? 'bg-primary/10 text-foreground'
            : hasError
              ? 'text-destructive/80 hover:bg-destructive/5'
              : 'text-muted-foreground hover:bg-muted/40',
        )}
        style={{ paddingLeft: `${depth * SECTION_TREE_INDENT_PX}px` }}
        draggable={dragIndex !== null}
        onDragStart={
          dragIndex === null ? undefined : (event) => reorder.onDragStart(event, dragIndex)
        }
        onDragOver={dragIndex === null ? undefined : reorder.onDragOver}
        onDrop={dragIndex === null ? undefined : (event) => reorder.onDrop(event, dragIndex)}
      >
        {isRoot && <GripVertical className="text-muted-foreground/60 h-3.5 w-3.5" />}
        <button
          type="button"
          className="flex flex-1 items-center gap-2 text-left"
          onClick={() => onSelectNode(node.id)}
        >
          <span className="text-[11px] font-medium">{label}</span>
          {hasError && (
            <span
              className="text-destructive flex items-center gap-1 text-[10px] font-semibold"
              title={`${errorCount} validation issue(s)`}
            >
              <AlertCircle className="h-3 w-3" />
              {errorCount}
            </span>
          )}
        </button>
        {dragIndex !== null && (
          <AddBlockButton
            anchorKey={nodeAnchorKey(node.id)}
            insertIndex={dragIndex + 1}
            label={`Add block after ${label}`}
            className={ROW_ACTION_CLASS}
          />
        )}
        <Button
          variant="ghost"
          size="icon"
          aria-label={`Remove ${label}`}
          className={ROW_ACTION_CLASS}
          onClick={() => onRemoveNode(node.id)}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </Button>
      </div>

      {slots.map(([slotKey, slotNode]) => (
        <div key={`${node.id}-slot-${slotKey}`} className="space-y-1">
          <div
            className="text-muted-foreground/70 text-[10px] font-semibold uppercase"
            style={{ paddingLeft: `${(depth + 1) * SECTION_TREE_INDENT_PX}px` }}
          >
            Slot: {slotKey}
          </div>
          <SectionTreeNode node={slotNode} depth={depth + 2} />
        </div>
      ))}

      {children.map((child) => (
        <SectionTreeNode key={child.id} node={child} depth={depth + 1} />
      ))}
    </div>
  )
}
