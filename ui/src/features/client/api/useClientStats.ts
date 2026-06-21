import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface ClientStats {
  total: number
  confidential: number
  public: number
}

export function useClientStats() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.clientStats(realm),
    queryFn: () => apiClient.get<ClientStats>(`/api/realms/${realm}/clients/stats`),
  })
}
