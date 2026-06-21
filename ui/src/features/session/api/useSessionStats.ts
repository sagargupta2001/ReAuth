import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface SessionStats {
  total_active: number
  unique_users: number
  active_last_24h: number
}

export function useSessionStats() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.sessionStats(realm),
    queryFn: () => apiClient.get<SessionStats>(`/api/realms/${realm}/sessions/stats`),
  })
}
