import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import { apiClient } from '@/shared/api/client'

export interface GroupChildrenParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useGroupChildrenList(groupId: string, params: GroupChildrenParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-children', realm, groupId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      return apiClient.get<PaginatedResponse<GroupTreeNode>>(
        `/api/realms/${realm}/rbac/groups/${groupId}/children?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
    enabled: !!groupId,
  })
}
