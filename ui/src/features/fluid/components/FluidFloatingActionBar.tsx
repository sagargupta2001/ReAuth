import { RotateCcw, RotateCw, SquareDashedMousePointer } from 'lucide-react'

import { Button } from '@/components/button'
import { cn } from '@/lib/utils'

interface FluidFloatingActionBarProps {
  isInspecting: boolean
  canUndo: boolean
  canRedo: boolean
  onUndo: () => void
  onRedo: () => void
  onToggleInspect: () => void
  className?: string
}

export function FluidFloatingActionBar({
  isInspecting,
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  onToggleInspect,
  className,
}: FluidFloatingActionBarProps) {
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
      >
        <RotateCcw className="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onClick={onRedo}
        disabled={!canRedo}
        className="h-8 w-8 rounded-full"
      >
        <RotateCw className="h-4 w-4" />
      </Button>
      <Button
        variant={isInspecting ? 'secondary' : 'outline'}
        size="icon"
        onClick={onToggleInspect}
        className="h-8 w-8 rounded-full"
      >
        <SquareDashedMousePointer className="h-4 w-4" />
      </Button>
    </div>
  )
}
