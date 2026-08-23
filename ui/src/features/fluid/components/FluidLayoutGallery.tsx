import { LAYOUT_SHELL_OPTIONS } from '@/features/fluid/model/layoutShells'
import { cn } from '@/lib/utils'

interface FluidLayoutGalleryProps {
  value?: string
  onChange: (value: string) => void
}

/** Picker for the structural shell a theme page renders inside. */
export function FluidLayoutGallery({ value, onChange }: FluidLayoutGalleryProps) {
  return (
    <div className="grid gap-3">
      {LAYOUT_SHELL_OPTIONS.map((layout) => {
        const Icon = layout.icon
        const isActive = value === layout.id

        return (
          <button
            key={layout.id}
            type="button"
            aria-pressed={isActive}
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
