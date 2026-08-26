import { useState } from 'react'

import { PopoverContent } from '@/components/popover'
import { BlockCatalogList } from '@/features/fluid/components/blocks/BlockCatalogList'
import { BlockPreview } from '@/features/fluid/components/blocks/BlockPreview'
import { useSectionsPanel } from '@/features/fluid/components/blocks/sectionsPanelContext'
import {
  FLUID_BLOCKS,
  findBlockDefinition,
  type FluidBlockId,
} from '@/features/fluid/model/blockCatalog'

interface BlockPickerProps {
  onSelectBlock: (blockId: FluidBlockId) => void
}

/**
 * Block picker popover: catalog on the left, live preview on the right.
 *
 * Search and hover state stay local so typing here never re-renders the tree.
 */
export function BlockPicker({ onSelectBlock }: BlockPickerProps) {
  const { drag, onClosePicker: onDragOutOfPicker } = useSectionsPanel()
  const [query, setQuery] = useState('')
  const [hoveredBlockId, setHoveredBlockId] = useState<FluidBlockId | null>(null)

  const previewBlock =
    (hoveredBlockId ? findBlockDefinition(hoveredBlockId) : undefined) ?? FLUID_BLOCKS[0]

  return (
    <PopoverContent
      align="start"
      side="right"
      sideOffset={12}
      collisionPadding={16}
      className="w-[560px] p-0 data-[state=closed]:hidden"
    >
      <div className="flex">
        <div className="w-2/5 border-r p-4">
          <BlockCatalogList
            query={query}
            onQueryChange={setQuery}
            onHoverBlock={setHoveredBlockId}
            onSelectBlock={onSelectBlock}
            onBlockDragStart={(event, blockId) => {
              drag.onBlockDragStart(event, blockId)
              // The picker is a popover; leaving it open would cover the very
              // canvas the block is being dragged onto.
              onDragOutOfPicker()
            }}
            onBlockDragEnd={drag.onDragEnd}
          />
        </div>

        <div className="w-3/5 p-4">
          <div className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
            Preview
          </div>
          <div className="bg-background rounded-lg border p-4 shadow-2xl">
            <div className="text-[11px] font-semibold">{previewBlock.label}</div>
            <p className="text-muted-foreground mb-4 text-[10px]">
              {previewBlock.description}
            </p>
            <BlockPreview blockId={previewBlock.id} />
          </div>
        </div>
      </div>
    </PopoverContent>
  )
}
