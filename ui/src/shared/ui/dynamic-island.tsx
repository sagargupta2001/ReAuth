import { type ComponentProps, type ReactNode } from 'react'

import { useFluidResize } from '@/lib/animations/useFluidResize'
import { cn } from '@/lib/utils'

type DynamicIslandProps = ComponentProps<'div'> & {
  /** Change this whenever the content changes to trigger the morph animation. */
  contentKey: string
  children: ReactNode
  ariaLabel?: string
}

/**
 * A content-agnostic pill that fluidly grows/shrinks as its content changes.
 * Owns the pill chrome (rounded, blurred, shadow-as-border) and clips overflow
 * during the morph so the transition reads as one continuous shape.
 */
export function DynamicIsland({
  contentKey,
  children,
  className,
  ariaLabel,
  ...props
}: DynamicIslandProps) {
  const ref = useFluidResize<HTMLDivElement>(contentKey)

  return (
    <div
      ref={ref}
      aria-label={ariaLabel}
      data-slot="dynamic-island"
      className={cn(
        'inline-flex max-w-full items-center overflow-hidden rounded-full border',
        'bg-background/70 supports-[backdrop-filter]:bg-background/55 shadow-sm backdrop-blur',
        'will-change-[width,height]',
        className,
      )}
      {...props}
    >
      <div className="flex shrink-0 items-center px-3 py-1.5">{children}</div>
    </div>
  )
}
