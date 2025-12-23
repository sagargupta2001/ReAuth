import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types.ts'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'

export interface UserSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useUsers(params: UserSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['users', realm, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      return apiClient.get<PaginatedResponse<User>>(
        `/api/realms/${realm}/users?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}
