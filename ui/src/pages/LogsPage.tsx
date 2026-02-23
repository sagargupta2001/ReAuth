import { useCallback, useMemo } from 'react'

import { useTranslation } from 'react-i18next'

import { ObservabilityLayout } from '@/features/observability/components/ObservabilityLayout'
import { MetricsOverview } from '@/features/observability/components/MetricsOverview'
import { LogsExplorer } from '@/features/observability/components/LogsExplorer'
import { TracesExplorer } from '@/features/observability/components/TracesExplorer'
import { CacheManager } from '@/features/observability/components/CacheManager'
import type { CustomTimeRange, TimeRangeKey } from '@/features/observability/lib/timeRange'
import { resolveTimeRange, TIME_RANGE_OPTIONS } from '@/features/observability/lib/timeRange'
import { enumParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'
import { Main } from '@/widgets/Layout/Main'

const TAB_OPTIONS = ['logs', 'traces', 'cache'] as const

export function LogsPage() {
  const { t } = useTranslation('logs')
  const timeRangeKeys = useMemo(
    () => TIME_RANGE_OPTIONS.map((option) => option.key) as TimeRangeKey[],
    [],
  )
  const [urlState, setUrlState] = useUrlState<{
    tab: (typeof TAB_OPTIONS)[number]
    range: TimeRangeKey
    start: string
    end: string
    trace: string
  }>({
    tab: enumParam(TAB_OPTIONS, 'logs'),
    range: enumParam(timeRangeKeys, '15m'),
    start: stringParam(''),
    end: stringParam(''),
    trace: stringParam(''),
  })

  const timeRangeKey = urlState.range
  const customRange: CustomTimeRange = {
    start: urlState.start || undefined,
    end: urlState.end || undefined,
  }
  const activeTab = urlState.tab
  const selectedTraceId = urlState.trace ? urlState.trace : null

  const resolvedTimeRange = useMemo(
    () => resolveTimeRange(timeRangeKey, customRange, new Date()),
    [timeRangeKey, urlState.start, urlState.end],
  )

  const handleTraceSelect = useCallback((traceId: string) => {
    setUrlState({ trace: traceId, tab: 'traces' })
  }, [setUrlState])

  const tabs = [
    {
      value: 'logs',
      label: t('OBSERVABILITY.TABS.LOGS'),
      content: (
        <LogsExplorer timeRange={resolvedTimeRange} onSelectTrace={handleTraceSelect} />
      ),
    },
    {
      value: 'traces',
      label: t('OBSERVABILITY.TABS.TRACES'),
      content: (
        <TracesExplorer
          timeRange={resolvedTimeRange}
          selectedTraceId={selectedTraceId}
          onSelectTrace={(traceId) => setUrlState({ trace: traceId })}
        />
      ),
    },
    {
      value: 'cache',
      label: t('OBSERVABILITY.TABS.CACHE'),
      content: <CacheManager />,
    },
  ]

  return (
    <Main fixed className="flex h-full flex-col p-12">
      <ObservabilityLayout
        title={t('OBSERVABILITY.TITLE')}
        description={t('OBSERVABILITY.DESCRIPTION')}
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={(value) => setUrlState({ tab: value as (typeof TAB_OPTIONS)[number] })}
        timeRange={timeRangeKey}
        onTimeRangeChange={(value) => {
          setUrlState({
            range: value,
            start: value === 'custom' ? urlState.start : '',
            end: value === 'custom' ? urlState.end : '',
          })
        }}
        customRange={customRange}
        onCustomRangeChange={(value) =>
          setUrlState({ start: value.start ?? '', end: value.end ?? '' })
        }
        timeRangeLabel={t('OBSERVABILITY.TIME_RANGE_LABEL')}
        timeRangePlaceholder={t('OBSERVABILITY.TIME_RANGE_PLACEHOLDER')}
        summary={<MetricsOverview />}
      />
    </Main>
  )
}
