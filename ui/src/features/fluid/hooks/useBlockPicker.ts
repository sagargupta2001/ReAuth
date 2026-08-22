import { useCallback, useMemo, useState } from 'react'

/**
 * Owns "which anchor opened the block picker, and where does the block land".
 *
 * Keeping this out of the panel means the tree rows only need to say
 * "open at this anchor, insert at this index".
 */
export interface BlockPickerState {
  /** Anchor key the picker is currently attached to, or `null` when closed. */
  openKey: string | null
  isOpen: boolean
  /** Index the next selected block is inserted at. */
  insertIndex: number
  open: (anchorKey: string, insertIndex: number) => void
  close: () => void
}

export function useBlockPicker(): BlockPickerState {
  const [openKey, setOpenKey] = useState<string | null>(null)
  const [insertIndex, setInsertIndex] = useState(0)

  const open = useCallback((anchorKey: string, index: number) => {
    setInsertIndex(index)
    setOpenKey(anchorKey)
  }, [])

  const close = useCallback(() => {
    setOpenKey(null)
  }, [])

  return useMemo(
    () => ({ openKey, isOpen: openKey !== null, insertIndex, open, close }),
    [openKey, insertIndex, open, close],
  )
}
