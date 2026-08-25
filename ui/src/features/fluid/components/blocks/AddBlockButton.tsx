import { Plus, type LucideIcon } from 'lucide-react'

import { Button } from '@/components/button'
import { PopoverAnchor } from '@/components/popover'
import { useSectionsPanel } from '@/features/fluid/components/blocks/sectionsPanelContext'
import type { NodeLocation } from '@/features/fluid/lib/nodeUtils'

interface AddBlockButtonProps {
  /** Identifies this button so the picker can anchor itself to it. */
  anchorKey: string
  /** Address the picked block is inserted at. */
  location: NodeLocation
  className?: string
  iconClassName?: string
  icon?: LucideIcon
  label?: string
}

/**
 * Opens the block picker and doubles as its popover anchor while open.
 */
export function AddBlockButton({
  anchorKey,
  location,
  className,
  iconClassName = 'h-3.5 w-3.5',
  icon: Icon = Plus,
  label = 'Add block',
}: AddBlockButtonProps) {
  const { pickerOpenKey, onOpenPicker } = useSectionsPanel()

  const button = (
    <Button
      variant="ghost"
      size="icon"
      aria-label={label}
      className={className}
      onClick={() => onOpenPicker(anchorKey, location)}
    >
      <Icon className={iconClassName} />
    </Button>
  )

  if (pickerOpenKey !== anchorKey) {
    return button
  }
  return <PopoverAnchor asChild>{button}</PopoverAnchor>
}
