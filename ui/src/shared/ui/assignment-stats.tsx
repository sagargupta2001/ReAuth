import type { LucideIcon } from 'lucide-react'

import { cn } from '@/lib/utils'

export interface AssignmentMetric {
  label: string
  value: number
  icon: LucideIcon
  hint?: string
}

interface AssignmentStatsProps {
  metrics: AssignmentMetric[]
  className?: string
}

/**
 * Row of compact metric cards (label + count + icon) shown above assignment-style
 * data tables — role members/composites, group members/roles, etc.
 */
export function AssignmentStats({ metrics, className }: AssignmentStatsProps) {
  return (
    <div className={cn('grid grid-cols-1 gap-3 sm:grid-cols-3', className)}>
      {metrics.map(({ label, value, icon: Icon, hint }) => (
        <div
          key={label}
          className="border-border/60 bg-surface-elevated flex items-center gap-3 rounded-xl border px-4 py-3"
        >
          <div className="bg-muted text-muted-foreground flex size-9 shrink-0 items-center justify-center rounded-lg">
            <Icon className="size-4" />
          </div>
          <div className="min-w-0">
            <p className="text-muted-foreground text-xs font-medium">{label}</p>
            <p className="text-xl leading-tight font-semibold tabular-nums">{value}</p>
            {hint ? <p className="text-muted-foreground truncate text-xs">{hint}</p> : null}
          </div>
        </div>
      ))}
    </div>
  )
}
