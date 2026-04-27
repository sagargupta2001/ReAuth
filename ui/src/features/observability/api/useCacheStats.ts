import { keepPreviousData, useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { CacheStats } from '../model/types'

export function useCacheStats(namespace?: string) {
  return useQuery({
    queryKey: queryKeys.observabilityCacheStats(namespace),
    queryFn: async () => {
      if (namespace) {
        return apiClient.get<CacheStats>(
          `/api/system/observability/cache/stats?namespace=${encodeURIComponent(namespace)}`,
        )
      }
      return apiClient.get<CacheStats[]>(`/api/system/observability/cache/stats`)
    },
    placeholderData: keepPreviousData,
    staleTime: 10_000,
  })
}
