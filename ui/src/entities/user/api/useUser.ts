import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types'
import { apiClient } from '@/shared/api/client'

export function useUser(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['user', userId],
    queryFn: () => apiClient.get<User>(`/api/realms/${realm}/users/${userId}`),
    enabled: !!userId,
  })
}
