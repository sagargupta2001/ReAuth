import { useQuery } from '@tanstack/react-query'

import type { Group } from '@/entities/group/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export async function fetchGroup(realm: string, groupId: string) {
  return apiClient.get<Group>(`/api/realms/${realm}/rbac/groups/${groupId}`)
}

export function useGroup(groupId: string, options?: { enabled?: boolean }) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.group(realm, groupId),
    queryFn: () => fetchGroup(realm, groupId),
    enabled: options?.enabled ?? true,
  })
}
