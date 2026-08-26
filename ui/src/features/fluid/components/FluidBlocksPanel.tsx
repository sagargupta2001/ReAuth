import { useCallback, useEffect, useMemo, useState } from 'react'

import { Popover } from '@/components/popover'
import { Separator } from '@/components/separator'
import type { ThemeNode } from '@/entities/theme/model/types'
import { AddBlockButton } from '@/features/fluid/components/blocks/AddBlockButton'
import { BlockPicker } from '@/features/fluid/components/blocks/BlockPicker'
import { PageValidationSummary } from '@/features/fluid/components/blocks/PageValidationSummary'
import { SectionTree } from '@/features/fluid/components/blocks/SectionTree'
import {
  SectionsPanelProvider,
  type SectionsPanelContextValue,
} from '@/features/fluid/components/blocks/sectionsPanelContext'
import { useBlockPicker } from '@/features/fluid/hooks/useBlockPicker'
import type { FluidDragController } from '@/features/fluid/hooks/useFluidDrag'
import type { NodeLocation } from '@/features/fluid/lib/nodeUtils'
import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'
import { buildValidationIndex } from '@/features/fluid/lib/validationIndex'
import {
  buildFluidNode,
  findBlockDefinition,
  type FluidBlockId,
} from '@/features/fluid/model/blockCatalog'
import { HEADER_ANCHOR_KEY } from '@/features/fluid/model/sectionTree'

interface FluidBlocksPanelProps {
  nodes: ThemeNode[]
  selectedNodeId: string | null
  validationErrors?: ThemeValidationError[]
  onSelectNode: (nodeId: string) => void
  onInsertNode: (node: ThemeNode, location: NodeLocation) => void
  onRemoveNode: (nodeId: string) => void
  /** The builder's shared drag session, so the tree and canvas cannot diverge. */
  drag: FluidDragController
}

/**
 * Left sidebar listing the page's blocks, with the block picker attached.
 *
 * This component only wires state together — the tree, the picker, and the
 * validation banner each own their own rendering.
 */
export function FluidBlocksPanel({
  nodes,
  selectedNodeId,
  validationErrors = [],
  onSelectNode,
  onInsertNode,
  onRemoveNode,
  drag,
}: FluidBlocksPanelProps) {
  const picker = useBlockPicker()
  // Collapse is a way of reading a deep tree, not a property of the page, so it
  // lives here and never reaches the draft.
  const [collapsedNodeIds, setCollapsedNodeIds] = useState<ReadonlySet<string>>(
    () => new Set(),
  )

  const onToggleCollapse = useCallback((nodeId: string) => {
    setCollapsedNodeIds((previous) => {
      const next = new Set(previous)
      if (!next.delete(nodeId)) {
        next.add(nodeId)
      }
      return next
    })
  }, [])

  // Hovering a collapsed container during a drag opens it, so a drop is never
  // blind. Driven off the shared drop target rather than a hook callback, which
  // keeps collapse — a tree-only view concern — out of the drag session.
  const insideTargetId =
    drag.dropTarget?.intent === 'inside' && drag.dropTarget.isAllowed
      ? drag.dropTarget.nodeId
      : null
  useEffect(() => {
    if (!insideTargetId) return
    setCollapsedNodeIds((previous) => {
      if (!previous.has(insideTargetId)) return previous
      const next = new Set(previous)
      next.delete(insideTargetId)
      return next
    })
  }, [insideTargetId])
  const { byNodeId, pageErrors } = useMemo(
    () => buildValidationIndex(validationErrors),
    [validationErrors],
  )

  const handleSelectBlock = (blockId: FluidBlockId) => {
    const definition = findBlockDefinition(blockId)
    if (!definition) return
    onInsertNode(buildFluidNode(definition), picker.insertLocation)
    picker.close()
  }

  const panelContext: SectionsPanelContextValue = useMemo(
    () => ({
      selectedNodeId,
      errorsByNodeId: byNodeId,
      collapsedNodeIds,
      onSelectNode,
      onRemoveNode,
      onToggleCollapse,
      drag,
      pickerOpenKey: picker.openKey,
      onOpenPicker: picker.open,
      onClosePicker: picker.close,
    }),
    [
      selectedNodeId,
      byNodeId,
      collapsedNodeIds,
      onSelectNode,
      onRemoveNode,
      onToggleCollapse,
      drag,
      picker.openKey,
      picker.open,
      picker.close,
    ],
  )

  return (
    <SectionsPanelProvider value={panelContext}>
      <Popover
        open={picker.isOpen}
        onOpenChange={(open) => {
          if (!open) picker.close()
        }}
      >
        <aside className="bg-muted/10 flex w-80 flex-col border-r">
          <div className="bg-background flex items-center justify-between border-b px-4 py-3">
            <h3 className="text-sm font-semibold">Sections</h3>
            <AddBlockButton
              anchorKey={HEADER_ANCHOR_KEY}
              location={{ parentId: null, index: nodes.length }}
              iconClassName="h-4 w-4"
            />
          </div>

          <div className="flex-1 overflow-y-auto p-4">
            <PageValidationSummary errors={pageErrors} />
            <SectionTree nodes={nodes} />
          </div>
          <Separator />
        </aside>

        <BlockPicker onSelectBlock={handleSelectBlock} />
      </Popover>
    </SectionsPanelProvider>
  )
}
