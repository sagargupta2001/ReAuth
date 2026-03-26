import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Role } from '@/features/roles/api/useRoles.ts'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export async function fetchRole(realm: string, roleId: string) {
  return apiClient.get<Role>(`/api/realms/${realm}/rbac/roles/${roleId}`)
}

export function useRole(roleId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.role(realm, roleId),
    queryFn: () => fetchRole(realm, roleId),
    enabled: !!roleId,
  })
}
