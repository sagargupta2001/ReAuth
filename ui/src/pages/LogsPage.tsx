import { useCallback, useMemo, useState } from 'react'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { LogsExplorer } from '@/features/observability/components/LogsExplorer'
import { MetricsOverview } from '@/features/observability/components/MetricsOverview'
import type { CustomTimeRange, TimeRangeKey } from '@/features/observability/lib/timeRange'
import { TIME_RANGE_OPTIONS, resolveTimeRange } from '@/features/observability/lib/timeRange'
import { enumParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'
import { Main } from '@/widgets/Layout/Main'

export function LogsPage() {
  const navigate = useRealmNavigate()
  const timeRangeKeys = useMemo(
    () => TIME_RANGE_OPTIONS.map((option) => option.key) as TimeRangeKey[],
    [],
  )
  const [urlState] = useUrlState<{
    range: TimeRangeKey
    start: string
    end: string
  }>({
    range: enumParam(timeRangeKeys, '15m'),
    start: stringParam(''),
    end: stringParam(''),
  })

  const timeRangeKey = urlState.range
  const customRange: CustomTimeRange = useMemo(
    () => ({
      start: urlState.start || undefined,
      end: urlState.end || undefined,
    }),
    [urlState.end, urlState.start],
  )

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
      navigate(`/logs/${traceId}`)
    },
    [navigate],
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
    </Main>
  )
}
