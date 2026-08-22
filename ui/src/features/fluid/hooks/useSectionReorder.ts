import { useMemo } from 'react'
import type { DragEvent } from 'react'

import { SECTION_REORDER_MIME_TYPE } from '@/features/fluid/model/sectionTree'

export interface SectionReorderHandlers {
  onDragStart: (event: DragEvent<HTMLElement>, index: number) => void
  onDragOver: (event: DragEvent<HTMLElement>) => void
  onDrop: (event: DragEvent<HTMLElement>, index: number) => void
}

/** Drag-and-drop wiring for reordering root-level section nodes. */
export function useSectionReorder(
  onReorderNodes: (fromIndex: number, toIndex: number) => void,
): SectionReorderHandlers {
  return useMemo(
    () => ({
      onDragStart: (event, index) => {
        event.dataTransfer.setData(SECTION_REORDER_MIME_TYPE, index.toString())
        event.dataTransfer.effectAllowed = 'move'
      },
      onDragOver: (event) => {
        event.preventDefault()
        event.dataTransfer.dropEffect = 'move'
      },
      onDrop: (event, index) => {
        const payload = event.dataTransfer.getData(SECTION_REORDER_MIME_TYPE)
        const fromIndex = Number.parseInt(payload, 10)
        if (Number.isNaN(fromIndex)) return

        event.preventDefault()
        event.stopPropagation()

        if (fromIndex === index) return
        onReorderNodes(fromIndex, index)
      },
    }),
    [onReorderNodes],
  )
}
