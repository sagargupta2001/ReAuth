import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client'

interface UpdateUserPayload {
  username: string
}

export function useUpdateUser(userId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: UpdateUserPayload) => {
      return apiClient.put(`/api/users/${userId}`, data)
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
