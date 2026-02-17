import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Role } from '@/features/roles/api/useRoles.ts'
import { apiClient } from '@/shared/api/client'

export function useRole(roleId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['role', realm, roleId],
    queryFn: async () => {
      return apiClient.get<Role>(`/api/realms/${realm}/rbac/roles/${roleId}`)
    },
    enabled: !!roleId,
  })
}
