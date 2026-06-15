import { Activity, Gauge, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { cn } from '@/lib/utils'

import { useMetricsSnapshot } from '../api/useMetricsSnapshot'

function formatDuration(value: number) {
  if (!Number.isFinite(value)) return '0ms'
  if (value >= 1000) {
    return `${(value / 1000).toFixed(2)}s`
  }
  return `${value.toFixed(0)}ms`
}

function formatPercent(value: number) {
  if (!Number.isFinite(value)) return '0%'
  return `${(value * 100).toFixed(2)}%`
}

export function MetricsOverview() {
  const { t } = useTranslation('logs')
  const { data } = useMetricsSnapshot()

  const requestCount = data?.request_count ?? 0
  const avgLatency = data?.latency_ms?.avg_ms ?? 0
  const serverErrors = data?.status_counts?.server_error ?? 0
  const errorRate = requestCount ? serverErrors / requestCount : 0
  const since = data?.since

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title={t('METRICS.REQUESTS')}
        value={requestCount.toLocaleString()}
        description={
          since ? t('METRICS.SINCE', { since: new Date(since).toLocaleString() }) : undefined
        }
        icon={Activity}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title={t('METRICS.AVG_LATENCY')}
        value={formatDuration(avgLatency)}
        description={t('METRICS.HISTOGRAM')}
        icon={Gauge}
        accentClassName="from-amber-500/20 via-amber-500/10 to-transparent"
        iconClassName="bg-amber-500/15 text-amber-500 ring-amber-500/20"
        barClassName="bg-amber-500"
      />
      <MetricCard
        title={t('METRICS.ERROR_RATE')}
        value={formatPercent(errorRate)}
        description={t('METRICS.SERVER_ERRORS', { count: serverErrors })}
        icon={ShieldAlert}
        accentClassName="from-rose-500/20 via-rose-500/10 to-transparent"
        iconClassName="bg-rose-500/15 text-rose-500 ring-rose-500/20"
        barClassName="bg-rose-500"
      />
    </div>
  )
}

type MetricCardProps = {
  title: string
  value: string
  description?: string
  icon: typeof Activity
  accentClassName: string
  iconClassName: string
  barClassName: string
}

function MetricCard({
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
