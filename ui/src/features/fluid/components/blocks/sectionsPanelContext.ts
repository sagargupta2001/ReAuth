import { createContext, useContext } from 'react'

import type { SectionDragHandlers } from '@/features/fluid/hooks/useSectionDrag'
import type { NodeLocation } from '@/features/fluid/lib/nodeUtils'
import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'

/**
 * Shared state for the sections panel tree.
 *
 * The tree is recursive, so passing selection/removal/drag callbacks down by
 * prop would mean threading the same seven props through every level.
 */
export interface SectionsPanelContextValue {
  selectedNodeId: string | null
  errorsByNodeId: ReadonlyMap<string, ThemeValidationError[]>
  /** View-only, and deliberately not persisted to the draft. */
  collapsedNodeIds: ReadonlySet<string>
  onSelectNode: (nodeId: string) => void
  onRemoveNode: (nodeId: string) => void
  onToggleCollapse: (nodeId: string) => void
  drag: SectionDragHandlers
  /** Anchor key the block picker is currently attached to. */
  pickerOpenKey: string | null
  onOpenPicker: (anchorKey: string, location: NodeLocation) => void
}

const SectionsPanelContext = createContext<SectionsPanelContextValue | null>(null)

export const SectionsPanelProvider = SectionsPanelContext.Provider

export function useSectionsPanel(): SectionsPanelContextValue {
  const value = useContext(SectionsPanelContext)
  if (!value) {
    throw new Error('useSectionsPanel must be used inside <SectionsPanelProvider>.')
  }
  return value
}
