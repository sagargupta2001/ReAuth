import { useQuery } from '@tanstack/react-query'

import type { User } from '@/entities/user/model/types'
import { apiClient } from '@/shared/api/client'

export function useUser(userId: string) {
  return useQuery({
    queryKey: ['user', userId],
    queryFn: () => apiClient.get<User>(`/api/users/${userId}`),
    enabled: !!userId,
  })
}
