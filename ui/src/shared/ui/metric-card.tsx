import type { ElementType } from 'react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { cn } from '@/lib/utils'

export type MetricCardProps = {
  title: string
  value: string | number
  description?: string
  icon: ElementType
  accentClassName: string
  iconClassName: string
  barClassName: string
}

export function MetricCard({
  title,
  value,
  description,
  icon: Icon,
  accentClassName,
  iconClassName,
  barClassName,
}: MetricCardProps) {
  return (
    <Card className="border-border/70 bg-surface-elevated relative overflow-hidden border">
      <div className={cn('absolute inset-x-0 top-0 h-20 bg-linear-to-b', accentClassName)} />
      <div className={cn('absolute inset-x-0 top-0 h-1', barClassName)} />
      <CardHeader className="relative flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-muted-foreground text-sm font-medium tracking-normal">
          {title}
        </CardTitle>
        <div className={cn('rounded-full p-2 ring-1', iconClassName)}>
          <Icon className="h-4 w-4" />
        </div>
      </CardHeader>
      <CardContent className="relative px-6">
        <div className="text-foreground text-3xl font-semibold tracking-tight">{value}</div>
        {description ? <p className="text-muted-foreground mt-1 text-xs">{description}</p> : null}
      </CardContent>
    </Card>
  )
}
