import { keepPreviousData, useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import type { TelemetryTrace } from '../model/types'

export interface TelemetryTraceQuery {
  search?: string
  start?: string
  end?: string
  page?: number
  per_page?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useTelemetryTraces(params: TelemetryTraceQuery) {
  return useQuery({
    queryKey: ['observability-traces', params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.search) query.set('search', params.search)
      if (params.start) query.set('start', params.start)
      if (params.end) query.set('end', params.end)
      if (params.page) query.set('page', String(params.page))
      if (params.per_page) query.set('per_page', String(params.per_page))
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      const queryString = query.toString()
      const url = queryString
        ? `/api/system/observability/traces?${queryString}`
        : '/api/system/observability/traces'
      return apiClient.get<PaginatedResponse<TelemetryTrace>>(url)
    },
    placeholderData: keepPreviousData,
    staleTime: 5_000,
  })
}
