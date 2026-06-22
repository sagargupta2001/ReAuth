import { Loader2, RotateCcw, RotateCw, Save, SquareDashedMousePointer } from 'lucide-react'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { cn } from '@/lib/utils'

interface BuilderFloatingActionBarProps {
  canUndo: boolean
  canRedo: boolean
  onUndo: () => void
  onRedo: () => void
  onSave: () => void
  isSaving?: boolean
  /** When provided, renders an inspect toggle between the history controls and Save. */
  isInspecting?: boolean
  onToggleInspect?: () => void
  className?: string
}

export function BuilderFloatingActionBar({
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  onSave,
  isSaving,
  isInspecting,
  onToggleInspect,
  className,
}: BuilderFloatingActionBarProps) {
  return (
    <div
      className={cn(
        'bg-background/80 fixed bottom-6 left-1/2 z-40 mx-auto flex w-fit -translate-x-1/2 items-center gap-2 rounded-full border p-2 px-3 shadow-lg backdrop-blur-md',
        className,
      )}
    >
      <Button
        variant="ghost"
        size="icon"
        onClick={onUndo}
        disabled={!canUndo}
        className="h-8 w-8 rounded-full"
        aria-label="Undo"
      >
        <RotateCcw className="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onClick={onRedo}
        disabled={!canRedo}
        className="h-8 w-8 rounded-full"
        aria-label="Redo"
      >
        <RotateCw className="h-4 w-4" />
      </Button>

      {onToggleInspect && (
        <Button
          variant={isInspecting ? 'secondary' : 'ghost'}
          size="icon"
          onClick={onToggleInspect}
          className="h-8 w-8 rounded-full"
          aria-label="Toggle inspect"
        >
          <SquareDashedMousePointer className="h-4 w-4" />
        </Button>
      )}

      <Separator orientation="vertical" className="mx-1 h-5" />

      <Button
        size="sm"
        onClick={onSave}
        disabled={isSaving}
        className="h-8 rounded-full px-4"
      >
        {isSaving ? (
          <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
        ) : (
          <Save className="mr-2 h-3.5 w-3.5" />
        )}
        Save Draft
      </Button>
    </div>
  )
}
