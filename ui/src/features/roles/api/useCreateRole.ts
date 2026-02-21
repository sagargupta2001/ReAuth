import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { RoleFormValues } from '@/features/roles/schema/create.schema.ts'
import { apiClient } from '@/shared/api/client'

type CreateRolePayload = RoleFormValues & { client_id?: string }

export function useCreateRole() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()

  return useMutation({
    mutationFn: async (data: CreateRolePayload) => {
      // The backend uses the same endpoint for creation
      return apiClient.post<{ id: string }>(`/api/realms/${realm}/rbac/roles`, data)
    },
    onSuccess: (data, variables) => {
      toast.success('Role created successfully')


      if (variables.client_id) {
        // Invalidate Client Roles list
        // Note: Ensure your useRoles hook uses this query key structure when clientId is present
        void queryClient.invalidateQueries({
          queryKey: ['roles', realm, { clientId: variables.client_id }],
        })
        // Redirect to Client Role Details (Optional: adjust path as needed)
        navigate(`/clients/${variables.client_id}/roles`)
      } else {
        // Invalidate Global Roles list
        void queryClient.invalidateQueries({ queryKey: ['roles', realm] })
        navigate(`/roles/${data.id}`)
      }
    },
    onError: () => {
      toast.error('Failed to create role')
    },
  })
}
