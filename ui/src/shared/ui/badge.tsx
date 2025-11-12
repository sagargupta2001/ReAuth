import * as React from 'react'

import { type VariantProps, cva } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80',
        secondary:
          'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive:
          'border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80',
        outline: 'border border-input text-foreground hover:bg-accent hover:text-accent-foreground',

        // 🟢 Success / Approved
        success: 'border-transparent bg-green-500 text-white shadow hover:bg-green-600',

        // 🔵 Info / Neutral
        info: 'border-transparent bg-blue-500 text-white shadow hover:bg-blue-600',

        // 🟡 Warning / Pending
        warning: 'border-transparent bg-yellow-400 text-yellow-900 shadow hover:bg-yellow-500',

        // ⚫ Muted / Subtle
        muted: 'border-transparent bg-muted text-muted-foreground hover:bg-muted/80',

        // 🟣 Purple for creative / special states
        purple: 'border-transparent bg-purple-500 text-white shadow hover:bg-purple-600',

        // 🌸 Pink for fun / playful tone
        pink: 'border-transparent bg-pink-500 text-white shadow hover:bg-pink-600',

        // 🧊 Cool blue outline style
        cool: 'border border-blue-300 bg-blue-50 text-blue-700 hover:bg-blue-100',

        // 🧡 Orange variant (for important notices)
        orange: 'border-transparent bg-orange-500 text-white shadow hover:bg-orange-600',
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
