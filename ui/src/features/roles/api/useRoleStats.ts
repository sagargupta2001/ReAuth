import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface RoleStats {
  total: number
  composite: number
  client: number
}

export function useRoleStats() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.roleStats(realm),
    queryFn: () => apiClient.get<RoleStats>(`/api/realms/${realm}/rbac/roles/stats`),
  })
}
