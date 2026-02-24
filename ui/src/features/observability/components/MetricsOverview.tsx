import { Activity, Gauge, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'

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
      <Card className="border-sky-500/30 bg-sky-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">
            {t('METRICS.REQUESTS')}
          </CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{requestCount}</div>
          {since && (
            <p className="text-xs text-muted-foreground">
              {t('METRICS.SINCE', { since: new Date(since).toLocaleString() })}
            </p>
          )}
        </CardContent>
      </Card>
      <Card className="border-emerald-500/30 bg-emerald-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">
            {t('METRICS.AVG_LATENCY')}
          </CardTitle>
          <Gauge className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{formatDuration(avgLatency)}</div>
          <p className="text-xs text-muted-foreground">
            {t('METRICS.HISTOGRAM')}
          </p>
        </CardContent>
      </Card>
      <Card className="border-amber-500/30 bg-amber-500/5">
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm text-muted-foreground">
            {t('METRICS.ERROR_RATE')}
          </CardTitle>
          <ShieldAlert className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-semibold">{formatPercent(errorRate)}</div>
          <p className="text-xs text-muted-foreground">
            {t('METRICS.SERVER_ERRORS', { count: serverErrors })}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
