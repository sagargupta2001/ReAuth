import { Activity, Gauge, Puzzle, ShieldCheck } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'

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
  const activePlugins = data?.active_plugins ?? 0
  const avgLatency = data?.avg_latency_ms ?? null
  const windowHours = data?.window_hours ?? 24

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <Card className="border-sky-500/30 bg-sky-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">
            Total Routed ({windowHours}h)
          </CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{totalRouted.toLocaleString()}</div>
          <p className="text-xs text-muted-foreground">Across webhooks and plugins</p>
        </CardContent>
      </Card>
      <Card className="border-emerald-500/30 bg-emerald-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">Delivery Success</CardTitle>
          <ShieldCheck className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{formatPercent(successRate)}</div>
          <p className="text-xs text-muted-foreground">Last {windowHours} hours</p>
        </CardContent>
      </Card>
      <Card className="border-amber-500/30 bg-amber-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">Active Plugins</CardTitle>
          <Puzzle className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{activePlugins}</div>
          <p className="text-xs text-muted-foreground">gRPC delivery targets</p>
        </CardContent>
      </Card>
      <Card className="border-indigo-500/30 bg-indigo-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">Avg. Latency</CardTitle>
          <Gauge className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{formatDuration(avgLatency)}</div>
          <p className="text-xs text-muted-foreground">Delivery pipeline average</p>
        </CardContent>
      </Card>
    </div>
  )
}
