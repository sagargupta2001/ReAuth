import { useMemo } from 'react'

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
import { useSectionReorder } from '@/features/fluid/hooks/useSectionReorder'
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
  onInsertNode: (node: ThemeNode, index: number) => void
  onRemoveNode: (nodeId: string) => void
  onReorderNodes: (fromIndex: number, toIndex: number) => void
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
  onReorderNodes,
}: FluidBlocksPanelProps) {
  const picker = useBlockPicker()
  const reorder = useSectionReorder(onReorderNodes)
  const { byNodeId, pageErrors } = useMemo(
    () => buildValidationIndex(validationErrors),
    [validationErrors],
  )

  const handleSelectBlock = (blockId: FluidBlockId) => {
    const definition = findBlockDefinition(blockId)
    if (!definition) return
    onInsertNode(buildFluidNode(definition), picker.insertIndex)
    picker.close()
  }

  const panelContext: SectionsPanelContextValue = useMemo(
    () => ({
      selectedNodeId,
      errorsByNodeId: byNodeId,
      onSelectNode,
      onRemoveNode,
      reorder,
      pickerOpenKey: picker.openKey,
      onOpenPicker: picker.open,
    }),
    [selectedNodeId, byNodeId, onSelectNode, onRemoveNode, reorder, picker.openKey, picker.open],
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
              insertIndex={nodes.length}
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
