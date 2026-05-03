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
        secondary:
          'bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80',
        destructive:
          'bg-destructive text-destructive-foreground shadow hover:bg-destructive/80',
        outline: 'shadow-sm text-foreground hover:bg-accent hover:text-accent-foreground',

        // 🟢 Success / Approved
        success: 'bg-green-500 text-white shadow hover:bg-green-600',

        // 🔵 Info / Neutral
        info: 'bg-blue-500/10 text-blue-500 hover:bg-blue-500/20',

        // 🟡 Warning / Pending
        warning: 'bg-yellow-400/10 text-yellow-500 hover:bg-yellow-400/20',

        // ⚫ Muted / Subtle
        muted: 'bg-muted text-muted-foreground hover:bg-muted/80',

        // 🟣 Purple for creative / special states
        purple: 'bg-purple-500 text-white shadow hover:bg-purple-600',

        // 🌸 Pink for fun / playful tone
        pink: 'bg-pink-500/10 text-pink-500 hover:bg-pink-500/20',

        // 🧊 Cool blue outline style
        cool: 'bg-blue-500/10 text-blue-500 hover:bg-blue-500/20',

        // 🧡 Orange variant (for important notices)
        orange: 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />
}

export { Badge, badgeVariants }