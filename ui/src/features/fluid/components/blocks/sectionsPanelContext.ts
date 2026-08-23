import { createContext, useContext } from 'react'

import type { SectionReorderHandlers } from '@/features/fluid/hooks/useSectionReorder'
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
  onSelectNode: (nodeId: string) => void
  onRemoveNode: (nodeId: string) => void
  reorder: SectionReorderHandlers
  /** Anchor key the block picker is currently attached to. */
  pickerOpenKey: string | null
  onOpenPicker: (anchorKey: string, insertIndex: number) => void
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
