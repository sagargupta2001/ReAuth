import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import type { RoleFormValues } from '@/features/roles/schema/create.schema.ts'



export function useCreateRole() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()

  return useMutation({
    mutationFn: async (data: RoleFormValues) => {
      return apiClient.post(`/api/realms/${realm}/rbac/roles`, data)
    },
    onSuccess: (data: any) => {
      toast.success('Role created successfully')
      void queryClient.invalidateQueries({ queryKey: ['roles', realm] })
      // Redirect to edit page to assign permissions immediately
      navigate(`/access/roles/${data.id}`)
    },
    onError: () => {
      toast.error('Failed to create role')
    },
  })
}
