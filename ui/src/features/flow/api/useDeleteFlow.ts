import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useDeleteFlow(flowId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!realm || !flowId) throw new Error('Missing Realm or Flow ID')
      return apiClient.delete(`/api/realms/${realm}/flows/${flowId}`)
    },
    onSuccess: () => {
      toast.success('Flow deleted.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.flows(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to delete flow.')
    },
  })
}
