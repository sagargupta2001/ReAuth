import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useDeleteUsers(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (userIds: string[]) => {
      const response = await apiClient.delete<{ status: string; count: number }>(
        `/api/realms/${realmId}/users`,
        { body: JSON.stringify({ user_ids: userIds }) },
      )
      return response
    },
    onSuccess: (data) => {
      toast.success(`Successfully deleted ${data.count} user${data.count === 1 ? '' : 's'}`)
      // Force generic invalidation for all users queries to ensure table refreshes
      void queryClient.invalidateQueries({ queryKey: queryKeys.users() })
    },
    onError: (error) => {
      console.error('Failed to delete users', error)
      toast.error('Failed to delete users. Please try again.')
    },
  })
}
