import { AlertCircle, ChevronDown, ChevronRight, CornerDownRight, GripVertical, Trash2 } from 'lucide-react'

import { Button } from '@/components/button'
import type { ThemeNode } from '@/entities/theme/model/types'
import { AddBlockButton } from '@/features/fluid/components/blocks/AddBlockButton'
import { useSectionsPanel } from '@/features/fluid/components/blocks/sectionsPanelContext'
import { canAcceptChildren, labelForNode } from '@/features/fluid/model/blockCatalog'
import {
  SECTION_TREE_INDENT_PX,
  insideAnchorKey,
  nodeAnchorKey,
} from '@/features/fluid/model/sectionTree'
import { cn } from '@/lib/utils'

const ROW_ACTION_CLASS = 'h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100'

interface SectionTreeNodeProps {
  node: ThemeNode
  depth: number
  /** Id of the authored parent, or `null` for a page-root node. */
  parentId?: string | null
  /** Position among the parent's children. */
  index?: number
  /**
   * Slot contents belong to a component's contract rather than to free
   * composition, so their rows are selectable but inert: not draggable, not
   * drop targets, and with no insertion affordance.
   */
  isSlotContent?: boolean
}

/** One row of the section tree, plus its slots and children. */
export function SectionTreeNode({
  node,
  depth,
  parentId = null,
  index = 0,
  isSlotContent = false,
}: SectionTreeNodeProps) {
  const {
    selectedNodeId,
    errorsByNodeId,
    collapsedNodeIds,
    onSelectNode,
    onRemoveNode,
    onToggleCollapse,
    drag,
  } = useSectionsPanel()

  const label = labelForNode(node)
  const isSelected = selectedNodeId === node.id
  const errorCount = errorsByNodeId.get(node.id)?.length ?? 0
  const hasError = errorCount > 0
  const slots = Object.entries(node.slots ?? {})
  const children = node.children ?? []
  const acceptsChildren = canAcceptChildren(node)
  const isInteractive = !isSlotContent
  const isCollapsible = children.length > 0 || slots.length > 0
  const isCollapsed = isCollapsible && collapsedNodeIds.has(node.id)
  const isDragging = drag.draggingNodeId === node.id
  const dropState = drag.dropTarget?.nodeId === node.id ? drag.dropTarget : null
  const isDropInside = dropState?.intent === 'inside'

  return (
    <div className="space-y-1">
      <div
        className={cn(
          'group relative flex items-center gap-2 rounded-md py-1 pr-2 transition-colors',
          isSelected
            ? 'bg-primary/10 text-foreground'
            : hasError
              ? 'text-destructive/80 hover:bg-destructive/5'
              : 'text-muted-foreground hover:bg-muted/40',
          isDragging && 'opacity-50',
          isDropInside &&
            (dropState.isAllowed
              ? 'ring-primary bg-primary/5 ring-1'
              : 'ring-destructive/60 ring-1'),
        )}
        style={{ paddingLeft: `${depth * SECTION_TREE_INDENT_PX}px` }}
        draggable={isInteractive}
        data-drop-intent={dropState?.intent}
        data-drop-allowed={dropState ? String(dropState.isAllowed) : undefined}
        onDragStart={isInteractive ? (event) => drag.onDragStart(event, node.id) : undefined}
        onDragEnd={isInteractive ? drag.onDragEnd : undefined}
        onDragOver={
          isInteractive
            ? (event) => drag.onRowDragOver(event, node.id, acceptsChildren)
            : undefined
        }
        onDragLeave={isInteractive ? (event) => drag.onDragLeave(event, node.id) : undefined}
        onDrop={
          isInteractive ? (event) => drag.onRowDrop(event, node.id, acceptsChildren) : undefined
        }
      >
        {dropState && !isDropInside && (
          <span
            aria-hidden="true"
            className={cn(
              'pointer-events-none absolute inset-x-2 h-0.5 rounded-full',
              dropState.isAllowed ? 'bg-primary' : 'bg-destructive',
              dropState.intent === 'before' ? 'top-0' : 'bottom-0',
            )}
          />
        )}
        {/*
          The chevron and grip columns are reserved on every row, not just the
          ones that use them. Rendering them conditionally removed ~22px of
          leading space from child rows, so a child's label sat further left
          than its parent's and the nesting read backwards.
        */}
        <span className="flex h-3.5 w-3.5 shrink-0 items-center justify-center">
          {isCollapsible && (
            <button
              type="button"
              aria-label={`${isCollapsed ? 'Expand' : 'Collapse'} ${label}`}
              aria-expanded={!isCollapsed}
              className="text-muted-foreground/70 hover:text-foreground"
              onClick={() => onToggleCollapse(node.id)}
            >
              {isCollapsed ? (
                <ChevronRight className="h-3.5 w-3.5" />
              ) : (
                <ChevronDown className="h-3.5 w-3.5" />
              )}
            </button>
          )}
        </span>
        <span className="flex h-3.5 w-3.5 shrink-0 items-center justify-center">
          {isInteractive && <GripVertical className="text-muted-foreground/60 h-3.5 w-3.5" />}
        </span>
        <button
          type="button"
          className="flex flex-1 items-center gap-2 text-left"
          onClick={() => onSelectNode(node.id)}
          onKeyDown={isInteractive ? (event) => drag.onRowKeyDown(event, node.id) : undefined}
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
        {isInteractive && acceptsChildren && (
          <AddBlockButton
            anchorKey={insideAnchorKey(node.id)}
            location={{ parentId: node.id, index: children.length }}
            icon={CornerDownRight}
            label={`Add block inside ${label}`}
            className={ROW_ACTION_CLASS}
          />
        )}
        {isInteractive && (
          <AddBlockButton
            anchorKey={nodeAnchorKey(node.id)}
            location={{ parentId, index: index + 1 }}
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

      {!isCollapsed &&
        slots.map(([slotKey, slotNode]) => (
          <div key={`${node.id}-slot-${slotKey}`} className="space-y-1">
            <div
              className="text-muted-foreground/70 text-[10px] font-semibold uppercase"
              style={{ paddingLeft: `${(depth + 1) * SECTION_TREE_INDENT_PX}px` }}
            >
              Slot: {slotKey}
            </div>
            <SectionTreeNode node={slotNode} depth={depth + 2} isSlotContent />
          </div>
        ))}

      {!isCollapsed && isInteractive && acceptsChildren && children.length === 0 && (
        <EmptyContainerDropZone node={node} depth={depth + 1} />
      )}

      {!isCollapsed &&
        children.map((child, childIndex) => (
          <SectionTreeNode
            key={child.id}
            node={child}
            depth={depth + 1}
            parentId={node.id}
            index={childIndex}
            isSlotContent={isSlotContent}
          />
        ))}
    </div>
  )
}

/**
 * A container with no children has no row to drop onto, so without this there
 * would be no way to put the first block into a new box.
 */
function EmptyContainerDropZone({ node, depth }: { node: ThemeNode; depth: number }) {
  const { drag } = useSectionsPanel()
  const dropState = drag.dropTarget?.nodeId === node.id ? drag.dropTarget : null
  const isActive = dropState?.intent === 'inside'

  return (
    <div
      data-empty-drop-zone={node.id}
      className={cn(
        'text-muted-foreground/60 rounded-md border border-dashed px-2 py-1 text-[10px]',
        isActive
          ? dropState.isAllowed
            ? 'border-primary text-primary bg-primary/5'
            : 'border-destructive text-destructive'
          : 'border-muted-foreground/25',
      )}
      style={{ marginLeft: `${depth * SECTION_TREE_INDENT_PX}px` }}
      onDragOver={(event) => drag.onInsideDragOver(event, node.id)}
      onDragLeave={(event) => drag.onDragLeave(event, node.id)}
      onDrop={(event) => drag.onInsideDrop(event, node.id)}
    >
      Drop a block here
    </div>
  )
}
