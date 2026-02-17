import { useQuery } from '@tanstack/react-query'

import type { Group } from '@/entities/group/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useGroup(groupId: string, options?: { enabled?: boolean }) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group', realm, groupId],
    queryFn: async () => {
      return apiClient.get<Group>(`/api/realms/${realm}/rbac/groups/${groupId}`)
    },
    enabled: options?.enabled ?? true,
  })
}
