import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export async function fetchUser(realm: string, userId: string) {
  return apiClient.get<User>(`/api/realms/${realm}/users/${userId}`)
}

export function useUser(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.user(userId),
    queryFn: () => fetchUser(realm, userId),
    enabled: !!userId,
  })
}
