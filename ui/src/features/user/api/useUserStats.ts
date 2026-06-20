import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface UserStats {
  total: number
  active_last_24h: number
  new_this_week: number
}

export function useUserStats() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.userStats(realm),
    queryFn: () => apiClient.get<UserStats>(`/api/realms/${realm}/users/stats`),
  })
}
