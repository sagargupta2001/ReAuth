import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'

export function useUser(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['user', userId],
    queryFn: () => apiClient.get<User>(`/api/realms/${realm}/users/${userId}`),
    enabled: !!userId,
  })
}
