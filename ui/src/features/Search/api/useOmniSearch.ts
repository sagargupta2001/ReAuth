import { keepPreviousData, useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

import type { OmniSearchResponse } from '@/features/Search/model/omniTypes'

export function useOmniSearch(query: string, limit = 6) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['omni-search', realm, query, limit],
    queryFn: async () => {
      const params = new URLSearchParams()
      params.set('q', query)
      params.set('limit', String(limit))
      return apiClient.get<OmniSearchResponse>(`/api/realms/${realm}/search?${params}`)
    },
    enabled: query.trim().length > 1,
    placeholderData: keepPreviousData,
    staleTime: 30_000,
  })
}
