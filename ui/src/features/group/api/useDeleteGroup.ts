import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

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
      void queryClient.invalidateQueries({ queryKey: queryKeys.groups(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.group(realm, groupId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.groupChildren(realm) })
    },
    onError: (err: unknown) => {
      const message = err instanceof Error ? err.message : 'Failed to delete group'
      toast.error(message)
    },
  })
}
