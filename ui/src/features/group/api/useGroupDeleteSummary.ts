import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface GroupDeleteSummary {
  group_id: string
  name: string
  direct_children_count: number
  descendant_count: number
  member_count: number
  role_count: number
}

export function useGroupDeleteSummary(groupId: string, enabled = false) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-delete-summary', realm, groupId],
    queryFn: async () => {
      return apiClient.get<GroupDeleteSummary>(
        `/api/realms/${realm}/rbac/groups/${groupId}/delete-summary`,
      )
    },
    enabled: enabled && !!groupId,
  })
}
