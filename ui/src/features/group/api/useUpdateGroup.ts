import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { GroupFormValues } from '@/features/group/schema/create.schema'
import { apiClient } from '@/shared/api/client'

export function useUpdateGroup(groupId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: async (data: GroupFormValues) => {
      return apiClient.put(`/api/realms/${realm}/rbac/groups/${groupId}`, data)
    },
    onSuccess: () => {
      toast.success('Group updated')
      void queryClient.invalidateQueries({ queryKey: ['groups', realm] })
      void queryClient.invalidateQueries({ queryKey: ['group', realm, groupId] })
    },
    onError: () => {
      toast.error('Failed to update group')
    },
  })
}
