import { useCallback, useMemo, useState } from 'react'

import { LogsExplorer } from '@/features/observability/components/LogsExplorer'
import { MetricsOverview } from '@/features/observability/components/MetricsOverview'
import { TraceWaterfallDialog } from '@/features/observability/components/TraceWaterfallDialog'
import type { CustomTimeRange, TimeRangeKey } from '@/features/observability/lib/timeRange'
import { TIME_RANGE_OPTIONS, resolveTimeRange } from '@/features/observability/lib/timeRange'
import { enumParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'
import { Main } from '@/widgets/Layout/Main'

export function LogsPage() {
  const timeRangeKeys = useMemo(
    () => TIME_RANGE_OPTIONS.map((option) => option.key) as TimeRangeKey[],
    [],
  )
  const [urlState, setUrlState] = useUrlState<{
    range: TimeRangeKey
    start: string
    end: string
    trace: string
  }>({
    range: enumParam(timeRangeKeys, '15m'),
    start: stringParam(''),
    end: stringParam(''),
    trace: stringParam(''),
  })

  const timeRangeKey = urlState.range
  const customRange: CustomTimeRange = useMemo(
    () => ({
      start: urlState.start || undefined,
      end: urlState.end || undefined,
    }),
    [urlState.end, urlState.start],
  )
  const selectedTraceId = urlState.trace ? urlState.trace : null

  const [now, setNow] = useState(() => new Date())

  const refreshTimeRange = useCallback(() => {
    if (timeRangeKey === 'custom') return
    setNow(new Date())
  }, [timeRangeKey])

  const resolvedTimeRange = useMemo(
    () => resolveTimeRange(timeRangeKey, customRange, now),
    [customRange, now, timeRangeKey],
  )

  const handleTraceSelect = useCallback(
    (traceId: string) => {
      setUrlState({ trace: traceId })
    },
    [setUrlState],
  )

  return (
    <Main fixed className="flex min-h-0 flex-1 flex-col p-6">
      <div className="flex min-h-0 flex-1 flex-col gap-4">
        <MetricsOverview />
        <LogsExplorer
          timeRange={resolvedTimeRange}
          onSelectTrace={handleTraceSelect}
          onRefreshTimeRange={refreshTimeRange}
        />
      </div>
      <TraceWaterfallDialog
        traceId={selectedTraceId}
        open={!!selectedTraceId}
        onOpenChange={(open) => {
          if (!open) setUrlState({ trace: '' })
        }}
      />
    </Main>
  )
}
