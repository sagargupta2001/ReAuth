import type { ElementType } from 'react'

import { CommandItem } from '@/components/command'
import { Switch } from '@/components/switch'
import { cn } from '@/lib/utils'

interface CommandSettingRowProps {
  value: string
  icon: ElementType
  label: string
  description?: string
  onSelect?: () => void
  onHighlight?: () => void
  toggle?: {
    checked: boolean
    onChange: (checked: boolean) => void
    ariaLabel?: string
    disabled?: boolean
  }
}

export function CommandSettingRow({
  value,
  icon: Icon,
  label,
  description,
  onSelect,
  onHighlight,
  toggle,
}: CommandSettingRowProps) {
  return (
    <CommandItem
      value={value}
      onSelect={onSelect}
      onFocus={onHighlight}
      onMouseEnter={onHighlight}
      className="py-2"
    >
      <div className="flex w-full items-center gap-3">
        <div className="bg-muted/60 text-muted-foreground flex h-8 w-8 items-center justify-center rounded-md">
          <Icon className="h-4 w-4" />
        </div>
        <div className="flex flex-1 flex-col">
          <span className="text-sm font-medium">{label}</span>
          {description && <span className="text-xs text-muted-foreground">{description}</span>}
        </div>
        {toggle && (
          <div
            className={cn('flex items-center')}
            onClick={(event) => event.stopPropagation()}
            onPointerDown={(event) => event.stopPropagation()}
          >
            <Switch
              checked={toggle.checked}
              onCheckedChange={toggle.onChange}
              aria-label={toggle.ariaLabel || label}
              disabled={toggle.disabled}
            />
          </div>
        )}
      </div>
    </CommandItem>
  )
}
