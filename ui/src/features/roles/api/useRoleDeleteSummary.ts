import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface RoleDeleteSummary {
  role_id: string
  name: string
  direct_user_count: number
  effective_user_count: number
  group_count: number
  parent_role_count: number
  child_role_count: number
  permission_count: number
}

export function useRoleDeleteSummary(roleId: string, enabled = false) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.roleDeleteSummary(realm, roleId),
    queryFn: async () => {
      return apiClient.get<RoleDeleteSummary>(
        `/api/realms/${realm}/rbac/roles/${roleId}/delete-summary`,
      )
    },
    enabled: enabled && !!roleId,
  })
}
