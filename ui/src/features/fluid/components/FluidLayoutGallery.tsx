import { LayoutTemplate, PanelRight, Square } from 'lucide-react'

import { cn } from '@/lib/utils'

export interface FluidLayoutOption {
  id: string
  name: string
  description: string
  icon: typeof LayoutTemplate
}

const layouts: FluidLayoutOption[] = [
  {
    id: 'CenteredCard',
    name: 'Centered Card',
    description: 'Classic centered form layout.',
    icon: LayoutTemplate,
  },
  {
    id: 'SplitScreen',
    name: 'Split Screen',
    description: 'Brand visual on the left, form on the right.',
    icon: PanelRight,
  },
  {
    id: 'Minimal',
    name: 'Minimal',
    description: 'Simple edge-to-edge layout.',
    icon: Square,
  },
]

interface FluidLayoutGalleryProps {
  value?: string
  onChange: (value: string) => void
}

export function FluidLayoutGallery({ value, onChange }: FluidLayoutGalleryProps) {
  return (
    <div className="grid gap-3">
      {layouts.map((layout) => {
        const Icon = layout.icon
        const isActive = value === layout.id

        return (
          <button
            key={layout.id}
            type="button"
            onClick={() => onChange(layout.id)}
            className={cn(
              'border-border hover:border-primary/60 hover:bg-muted/40 flex w-full items-center gap-3 rounded-lg border px-3 py-2 text-left transition-colors',
              isActive && 'border-primary bg-primary/5 shadow-sm',
            )}
          >
            <span
              className={cn(
                'bg-muted flex h-8 w-8 items-center justify-center rounded-md',
                isActive && 'bg-primary/10 text-primary',
              )}
            >
              <Icon className="h-4 w-4" />
            </span>
            <span className="flex flex-col">
              <span className="text-sm font-medium">{layout.name}</span>
              <span className="text-muted-foreground text-[11px]">{layout.description}</span>
            </span>
          </button>
        )
      })}
    </div>
  )
}
