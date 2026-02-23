import { keepPreviousData, useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import type { TelemetryLog } from '../model/types'

export interface TelemetryLogQuery {
  level?: string
  target?: string
  search?: string
  start?: string
  end?: string
  page?: number
  per_page?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useTelemetryLogs(params: TelemetryLogQuery) {
  return useQuery({
    queryKey: ['observability-logs', params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.level) query.set('level', params.level)
      if (params.target) query.set('target', params.target)
      if (params.search) query.set('search', params.search)
      if (params.start) query.set('start', params.start)
      if (params.end) query.set('end', params.end)
      if (params.page) query.set('page', String(params.page))
      if (params.per_page) query.set('per_page', String(params.per_page))
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      const queryString = query.toString()
      const url = queryString
        ? `/api/system/observability/logs?${queryString}`
        : '/api/system/observability/logs'
      return apiClient.get<PaginatedResponse<TelemetryLog>>(url)
    },
    placeholderData: keepPreviousData,
    staleTime: 5_000,
  })
}
