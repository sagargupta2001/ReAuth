import { useEffect, useRef } from 'react'

import gsap from 'gsap'
import { Loader2, RotateCcw, Save } from 'lucide-react'

import { Button } from '@/components/button'
import { cn } from '@/lib/utils'

interface FloatingActionBarProps {
  isOpen: boolean
  isPending?: boolean
  onSave: () => void
  onReset: () => void
  className?: string
}

export function FloatingActionBar({
  isOpen,
  isPending,
  onSave,
  onReset,
  className,
}: FloatingActionBarProps) {
  const barRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (!barRef.current) return

    if (isOpen) {
      // animate in
      gsap.fromTo(
        barRef.current,
        { y: 100, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: 0.35,
          ease: 'power3.out',
        },
      )
    } else {
      // animate out
      gsap.to(barRef.current, {
        y: 100,
        opacity: 0,
        duration: 0.25,
        ease: 'power3.in',
      })
    }
  }, [isOpen])

  if (!isOpen) {
    // do NOT render to DOM, GSAP handles exit animation before unmount happens
    return null
  }

  return (
    <div
      ref={barRef}
      className={cn(
        'bg-background/80 fixed right-0 bottom-6 left-0 z-50 mx-auto flex w-fit items-center gap-4 rounded-full border p-2 px-4 shadow-lg backdrop-blur-md',
        className,
      )}
    >
      <span className="text-muted-foreground ml-2 hidden text-sm font-medium sm:inline-block">
        Unsaved changes
      </span>

      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={onReset}
          disabled={isPending}
          className="text-muted-foreground hover:bg-muted h-8 rounded-full"
        >
          <RotateCcw className="mr-2 h-3.5 w-3.5" />
          Reset
        </Button>

        <Button size="sm" onClick={onSave} disabled={isPending} className="h-8 rounded-full px-4">
          {isPending ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Save className="mr-2 h-3.5 w-3.5" />
          )}
          Save Changes
        </Button>
      </div>
    </div>
  )
}
