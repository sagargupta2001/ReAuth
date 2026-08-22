import { Plus } from 'lucide-react'

import { Button } from '@/components/button'
import { PopoverAnchor } from '@/components/popover'
import { useSectionsPanel } from '@/features/fluid/components/blocks/sectionsPanelContext'

interface AddBlockButtonProps {
  /** Identifies this button so the picker can anchor itself to it. */
  anchorKey: string
  /** Position the picked block is inserted at. */
  insertIndex: number
  className?: string
  iconClassName?: string
  label?: string
}

/**
 * Opens the block picker and doubles as its popover anchor while open.
 */
export function AddBlockButton({
  anchorKey,
  insertIndex,
  className,
  iconClassName = 'h-3.5 w-3.5',
  label = 'Add block',
}: AddBlockButtonProps) {
  const { pickerOpenKey, onOpenPicker } = useSectionsPanel()

  const button = (
    <Button
      variant="ghost"
      size="icon"
      aria-label={label}
      className={className}
      onClick={() => onOpenPicker(anchorKey, insertIndex)}
    >
      <Plus className={iconClassName} />
    </Button>
  )

  if (pickerOpenKey !== anchorKey) {
    return button
  }
  return <PopoverAnchor asChild>{button}</PopoverAnchor>
}
