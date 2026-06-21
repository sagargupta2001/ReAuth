import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { User } from '@/entities/user/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useCurrentUser() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.currentUser(realm),
    queryFn: () => apiClient.get<User>(`/api/realms/${realm}/users/me`),
  })
}
