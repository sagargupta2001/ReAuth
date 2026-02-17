import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { apiClient } from '@/shared/api/client.ts'

interface UpdateUserPayload {
  username: string
}

export function useUpdateUser(userId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: UpdateUserPayload) => {
      return apiClient.put(`/api/realms/${realm}/users/${userId}`, data)
    },
    onSuccess: () => {
      toast.success('User updated successfully')
      void queryClient.invalidateQueries({ queryKey: ['user', userId] })
      void queryClient.invalidateQueries({ queryKey: ['users'] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update user')
    },
  })
}
