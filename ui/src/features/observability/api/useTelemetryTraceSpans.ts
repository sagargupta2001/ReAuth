import { keepPreviousData, useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { TelemetryTrace } from '../model/types'

export function useTelemetryTraceSpans(traceId?: string | null) {
  return useQuery({
    queryKey: queryKeys.observabilityTraceSpans(traceId ?? ''),
    queryFn: async () => {
      if (!traceId) return []
      return apiClient.get<TelemetryTrace[]>(`/api/system/observability/traces/${traceId}`)
    },
    enabled: !!traceId,
    placeholderData: keepPreviousData,
    staleTime: 5_000,
  })
}
