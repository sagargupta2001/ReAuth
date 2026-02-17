import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import type { RoleFormValues } from '@/features/roles/schema/create.schema.ts'


export function useUpdateRole(roleId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: async (data: RoleFormValues) => {
      // Assuming PUT /api/realms/{realm}/rbac/roles/{id} exists
      return apiClient.put(`/api/realms/${realm}/rbac/roles/${roleId}`, data)
    },
    onSuccess: () => {
      toast.success('Role updated successfully')
      void queryClient.invalidateQueries({ queryKey: ['roles', realm] })
      void queryClient.invalidateQueries({ queryKey: ['role', realm, roleId] })
    },
    onError: () => {
      toast.error('Failed to update role')
    },
  })
}
