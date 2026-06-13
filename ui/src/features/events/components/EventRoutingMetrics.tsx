import { Activity, Gauge, ShieldCheck } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { cn } from '@/lib/utils'

import { useEventRoutingMetrics } from '../api/useEventRoutingMetrics'

function formatDuration(value?: number | null) {
  if (!value || !Number.isFinite(value)) return '—'
  if (value >= 1000) {
    return `${(value / 1000).toFixed(2)}s`
  }
  return `${value.toFixed(0)}ms`
}

function formatPercent(value?: number | null) {
  if (value === null || value === undefined || !Number.isFinite(value)) return '—'
  return `${(value * 100).toFixed(2)}%`
}

export function EventRoutingMetrics() {
  const { data } = useEventRoutingMetrics()

  const totalRouted = data?.total_routed ?? 0
  const successRate = data?.success_rate ?? 0
  const avgLatency = data?.avg_latency_ms ?? null
  const windowHours = data?.window_hours ?? 24
  const successPercent = Math.max(0, Math.min(successRate * 100, 100))

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title={`Total Routed (${windowHours}h)`}
        value={totalRouted.toLocaleString()}
        description="Across webhook targets"
        icon={Activity}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Delivery Success"
        value={formatPercent(successRate)}
        description={`Last ${windowHours} hours`}
        icon={ShieldCheck}
        accentClassName="from-emerald-500/20 via-emerald-500/10 to-transparent"
        iconClassName="bg-emerald-500/15 text-emerald-500 ring-emerald-500/20"
        barClassName="bg-emerald-500"
        progress={successPercent}
      />
      <MetricCard
        title="Avg. Latency"
        value={formatDuration(avgLatency)}
        description="Delivery pipeline average"
        icon={Gauge}
        accentClassName="from-amber-500/20 via-amber-500/10 to-transparent"
        iconClassName="bg-amber-500/15 text-amber-500 ring-amber-500/20"
        barClassName="bg-amber-500"
      />
    </div>
  )
}

type MetricCardProps = {
  title: string
  value: string
  description: string
  icon: typeof Activity
  accentClassName: string
  iconClassName: string
  barClassName: string
  progress?: number
}

function MetricCard({
  title,
  value,
  description,
  icon: Icon,
  accentClassName,
  iconClassName,
  barClassName,
  progress,
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
      <CardContent className="relative px-6 ">
        <div className="text-foreground text-3xl font-semibold tracking-tight">{value}</div>
        <p className="text-muted-foreground mt-1 text-xs">{description}</p>
        {progress !== undefined ? (
          <div className="bg-muted mt-4 h-1.5 overflow-hidden rounded-full">
            <div
              className={cn('h-full rounded-full', barClassName)}
              style={{ width: `${progress}%` }}
            />
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}
