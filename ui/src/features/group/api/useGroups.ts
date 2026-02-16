import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Group } from '@/entities/group/model/types'
import { apiClient } from '@/shared/api/client'

export interface GroupSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useGroups(params: GroupSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['groups', realm, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      return apiClient.get<PaginatedResponse<Group>>(
        `/api/realms/${realm}/rbac/groups?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}
