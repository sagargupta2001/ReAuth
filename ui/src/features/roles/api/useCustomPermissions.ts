import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import type { PermissionDef } from '@/features/roles/api/usePermissions'

export interface CreateCustomPermissionPayload {
  permission: string
  name: string
  description?: string | null
  client_id?: string | null
}

export function useCreateCustomPermission() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: CreateCustomPermissionPayload) => {
      return apiClient.post<PermissionDef>(`/api/realms/${realm}/rbac/permissions/custom`, payload)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['permissions-definitions', realm] })
      toast.success('Custom permission created')
    },
    onError: () => toast.error('Failed to create permission'),
  })
}

export interface UpdateCustomPermissionPayload {
  name: string
  description?: string | null
}

export function useUpdateCustomPermission() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: { id: string; data: UpdateCustomPermissionPayload }) => {
      return apiClient.put<PermissionDef>(
        `/api/realms/${realm}/rbac/permissions/custom/${payload.id}`,
        payload.data,
      )
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['permissions-definitions', realm] })
      toast.success('Custom permission updated')
    },
    onError: () => toast.error('Failed to update permission'),
  })
}

export function useDeleteCustomPermission() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (id: string) => {
      return apiClient.delete(`/api/realms/${realm}/rbac/permissions/custom/${id}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['permissions-definitions', realm] })
      toast.success('Custom permission deleted')
    },
    onError: () => toast.error('Failed to delete permission'),
  })
}
