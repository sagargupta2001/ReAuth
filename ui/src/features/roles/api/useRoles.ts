import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types.ts'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { apiClient } from '@/shared/api/client.ts'

// Define Role Type locally or in shared entities
export interface Role {
  id: string
  name: string
  description?: string
  created_at?: string
  user_count?: number // Future proofing
}

export interface RoleSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export function useRoles(params: RoleSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['roles', realm, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      return apiClient.get<PaginatedResponse<Role>>(
        `/api/realms/${realm}/rbac/roles?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}
