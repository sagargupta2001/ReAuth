import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { GroupFormValues } from '@/features/group/schema/create.schema'
import type { Group } from '@/entities/group/model/types'
import { apiClient } from '@/shared/api/client'

export function useCreateGroup() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()

  return useMutation<Group, Error, GroupFormValues>({
    mutationFn: async (data: GroupFormValues) => {
      return apiClient.post<Group>(`/api/realms/${realm}/rbac/groups`, data)
    },
    onSuccess: (data: Group) => {
      toast.success('Group created successfully')
      void queryClient.invalidateQueries({ queryKey: ['groups', realm] })
      navigate(`/groups/${data.id}`)
    },
    onError: () => {
      toast.error('Failed to create group')
    },
  })
}
