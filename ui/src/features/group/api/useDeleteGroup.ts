import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

interface DeleteGroupPayload {
  cascade?: boolean
}

export function useDeleteGroup(groupId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ cascade }: DeleteGroupPayload) => {
      const query = cascade ? '?cascade=true' : ''
      return apiClient.delete(`/api/realms/${realm}/rbac/groups/${groupId}${query}`)
    },
    onSuccess: () => {
      toast.success('Group deleted')
      void queryClient.invalidateQueries({ queryKey: ['groups', realm] })
      void queryClient.invalidateQueries({ queryKey: ['group', realm, groupId] })
      void queryClient.invalidateQueries({ queryKey: ['group-children', realm] })
    },
    onError: (err: any) => {
      toast.error(err?.message || 'Failed to delete group')
    },
  })
}
