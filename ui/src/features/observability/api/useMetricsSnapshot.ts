import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { MetricsSnapshot } from '../model/types'

export function useMetricsSnapshot() {
  return useQuery({
    queryKey: queryKeys.observabilityMetrics(),
    queryFn: async () => apiClient.get<MetricsSnapshot>('/api/system/observability/metrics'),
    staleTime: 10_000,
    refetchInterval: 30_000,
  })
}
