import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface Role {
  id: string
  realm_id: string
  client_id?: string | null // Identify if it's a client role
  name: string
  description?: string
  created_at?: string
  user_count?: number
}

export interface RoleSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  clientId?: string
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

      // If clientId is present, fetch roles for that specific client.
      // Otherwise, fetch global realm roles.
      let url = `/api/realms/${realm}/rbac/roles`

      if (params.clientId) {
        url = `/api/realms/${realm}/rbac/clients/${params.clientId}/roles`
      }

      return apiClient.get<PaginatedResponse<Role>>(`${url}?${query.toString()}`)
    },
    placeholderData: keepPreviousData,
  })
}
