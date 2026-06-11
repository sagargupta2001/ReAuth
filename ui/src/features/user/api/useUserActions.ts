import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { User } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useLockUser(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => apiClient.post<User>(`/api/realms/${realm}/users/${userId}/lock`, {}),
    onSuccess: () => {
      toast.success('User locked.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.users() })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to lock user.')
    },
  })
}

export function useBanUser(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => apiClient.post<User>(`/api/realms/${realm}/users/${userId}/ban`, {}),
    onSuccess: () => {
      toast.success('User banned.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.users() })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to ban user.')
    },
  })
}

export function useDeleteUser(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () =>
      apiClient.raw(`/api/realms/${realm}/users`, {
        method: 'DELETE',
        body: JSON.stringify({ user_ids: [userId] }),
      }),
    onSuccess: () => {
      toast.success('User deleted.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.users() })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to delete user.')
    },
  })
}
