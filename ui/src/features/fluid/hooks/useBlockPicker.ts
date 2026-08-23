import { useCallback, useMemo, useState } from 'react'

import type { NodeLocation } from '@/features/fluid/lib/nodeUtils'

const ROOT_END: NodeLocation = { parentId: null, index: 0 }

/**
 * Owns "which anchor opened the block picker, and where does the block land".
 *
 * Keeping this out of the panel means the tree rows only need to say
 * "open at this anchor, insert at this address". The address is a parent id
 * plus a position, not a bare index, so "add inside this box" and "add after
 * this row" are distinct actions.
 */
export interface BlockPickerState {
  /** Anchor key the picker is currently attached to, or `null` when closed. */
  openKey: string | null
  isOpen: boolean
  /** Address the next selected block is inserted at. */
  insertLocation: NodeLocation
  open: (anchorKey: string, location: NodeLocation) => void
  close: () => void
}

export function useBlockPicker(): BlockPickerState {
  const [openKey, setOpenKey] = useState<string | null>(null)
  const [insertLocation, setInsertLocation] = useState<NodeLocation>(ROOT_END)

  const open = useCallback((anchorKey: string, location: NodeLocation) => {
    setInsertLocation(location)
    setOpenKey(anchorKey)
  }, [])

  const close = useCallback(() => {
    setOpenKey(null)
  }, [])

  return useMemo(
    () => ({ openKey, isOpen: openKey !== null, insertLocation, open, close }),
    [openKey, insertLocation, open, close],
  )
}
