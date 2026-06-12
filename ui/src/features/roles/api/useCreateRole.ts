import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { RoleFormValues } from '@/features/roles/schema/create.schema.ts'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

type CreateRolePayload = RoleFormValues & { client_id?: string }

export function useCreateRole() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: async (data: CreateRolePayload) => {
      return apiClient.post<{ id: string }>(`/api/realms/${realm}/rbac/roles`, data)
    },
    onSuccess: () => {
      toast.success('Role created successfully')
      void queryClient.invalidateQueries({ queryKey: queryKeys.roles(realm) })
    },
    onError: () => {
      toast.error('Failed to create role')
    },
  })
}
