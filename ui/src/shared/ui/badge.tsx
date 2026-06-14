/* eslint-disable react-refresh/only-export-components */
import * as React from 'react'

import { type VariantProps, cva } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center rounded-full px-2.5 py-0.5 text-[12px] font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground shadow-sm hover:bg-primary/80',
        secondary: 'bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80',
        destructive: 'bg-destructive text-destructive-foreground shadow hover:bg-destructive/80',
        outline: 'shadow-sm text-foreground hover:bg-accent hover:text-accent-foreground',
        success: 'bg-green-500 text-white shadow hover:bg-green-600',
        successMuted:
          'border border-emerald-500/25 bg-emerald-950/60 text-emerald-300 hover:bg-emerald-950/80',
        dangerMuted: 'border border-rose-500/25 bg-rose-950/60 text-rose-300 hover:bg-rose-950/80',
        warningMuted:
          'border border-amber-500/25 bg-amber-950/60 text-amber-300 hover:bg-amber-950/80',
        neutralMuted: 'border border-border/70 bg-muted/60 text-muted-foreground hover:bg-muted/80',
        info: 'bg-blue-500/10 text-blue-500 hover:bg-blue-500/20',
        warning: 'bg-yellow-400/10 text-yellow-500 hover:bg-yellow-400/20',
        muted: 'bg-muted text-muted-foreground hover:bg-muted/80',
        purple: 'bg-purple-500 text-white shadow hover:bg-purple-600',
        pink: 'bg-pink-500/10 text-pink-500 hover:bg-pink-500/20',
        cool: 'bg-blue-500/10 text-blue-500 hover:bg-blue-500/20',
        orange: 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant }), className)} {...props} />
}

export { Badge, badgeVariants }
