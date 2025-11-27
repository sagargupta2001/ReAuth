import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { ClientSearchParams, OidcClient, PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useClients(params: ClientSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    // Include params in the query key so it refetches when they change
    queryKey: ['clients', realm, params],

    queryFn: async () => {
      // Convert params object to URLSearchParams
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)

      return apiClient.get<PaginatedResponse<OidcClient>>(
        `/api/realms/${realm}/clients?${query.toString()}`,
      )
    },
    // Keep previous data while fetching new page (prevents table flicker)
    placeholderData: keepPreviousData,
  })
}
