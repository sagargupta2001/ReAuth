import { Activity, Gauge, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { MetricCard } from '@/shared/ui/metric-card'

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

