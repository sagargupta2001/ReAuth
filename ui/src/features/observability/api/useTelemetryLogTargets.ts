import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'

export interface TelemetryLogTargetsQuery {
  level?: string
  search?: string
  start?: string
  end?: string
  include_spans?: boolean
}

export function useTelemetryLogTargets(params: TelemetryLogTargetsQuery) {
  return useQuery({
    queryKey: ['observability-log-targets', params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.level) query.set('level', params.level)
      if (params.search) query.set('search', params.search)
      if (params.start) query.set('start', params.start)
      if (params.end) query.set('end', params.end)
      if (params.include_spans !== undefined) {
        query.set('include_spans', String(params.include_spans))
      }
      const queryString = query.toString()
      const url = queryString
        ? `/api/system/observability/logs/targets?${queryString}`
        : '/api/system/observability/logs/targets'
      return apiClient.get<string[]>(url)
    },
    staleTime: 10_000,
    refetchOnWindowFocus: false,
  })
}
