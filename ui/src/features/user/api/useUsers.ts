import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types.ts'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

import { serializeFilterValue, type DataTableFilterValue } from '@/shared/ui/data-table/types'

const supportedUserFilterKeys = new Set(['email', 'created_at', 'last_sign_in_at'])

export interface UserSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filters?: DataTableFilterValue[]
}

function hasMeaningfulFilterValue(value: unknown): boolean {
  if (value == null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (typeof value === 'object' && value !== null) {
    const rangeValue = value as { from?: unknown; to?: unknown }
    return rangeValue.from != null || rangeValue.to != null
  }
  return true
}

export function useUsers(params: UserSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.users(realm, params),
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      params.filters?.forEach((f) => {
        if (!supportedUserFilterKeys.has(f.key)) return
        if (!hasMeaningfulFilterValue(f.value)) return
        const value = serializeFilterValue(f.value)
        if (!value) return
        query.set(`filter_${f.key}`, value)
      })

      return apiClient.get<PaginatedResponse<User>>(
        `/api/realms/${realm}/users?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}
