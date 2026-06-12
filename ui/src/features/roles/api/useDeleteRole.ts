import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useDeleteRole(roleId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      return apiClient.delete(`/api/realms/${realm}/rbac/roles/${roleId}`)
    },
    onSuccess: () => {
      toast.success('Role deleted')
      void queryClient.invalidateQueries({ queryKey: queryKeys.roles(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.role(realm, roleId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.roleDeleteSummary(realm, roleId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.rolePermissions(realm, roleId) })
    },
    onError: (error: unknown) => {
      const message = error instanceof Error ? error.message : 'Failed to delete role'
      toast.error(message)
    },
  })
}
