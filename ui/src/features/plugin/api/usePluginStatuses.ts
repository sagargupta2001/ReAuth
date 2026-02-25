import { useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import type { PluginStatusInfo } from '@/entities/plugin/model/types'

export interface PluginStatusParams {
  page?: number
  per_page?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  q?: string
}

export function usePluginStatuses(params: PluginStatusParams) {
  return useQuery({
    queryKey: ['plugin-statuses', params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.page) query.set('page', String(params.page))
      if (params.per_page) query.set('per_page', String(params.per_page))
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.q) query.set('q', params.q)
      const queryString = query.toString()
      const url = queryString ? `/api/plugins/statuses?${queryString}` : '/api/plugins/statuses'
      const res = await fetch(url)
      if (!res.ok) {
        throw new Error(`Failed to fetch plugin statuses: ${res.statusText}`)
      }
      return (await res.json()) as PaginatedResponse<PluginStatusInfo>
    },
  })
}
