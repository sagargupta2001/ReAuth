import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

// 1. Hook to Fetch Assigned Permissions
export function useRolePermissions(roleId: string) {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: ['role-permissions', realm, roleId],
    queryFn: async () => {
      // Direct array return from our new backend endpoint
      return apiClient.get<string[]>(`/api/realms/${realm}/rbac/roles/${roleId}/permissions`)
    },
  })
}

// 2. Hook to Manage Permissions (Single & Bulk)
export function useManagePermissions(roleId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const queryKey = ['role-permissions', realm, roleId]

  // Single Toggle Mutation
  const toggleMutation = useMutation({
    mutationFn: async ({ permission, action }: { permission: string; action: 'add' | 'remove' }) => {
      if (action === 'add') {
        return apiClient.post(`/api/realms/${realm}/rbac/roles/${roleId}/permissions`, { permission })
      } else {
        // Using HTTP Delete with body
        return apiClient.delete(`/api/realms/${realm}/rbac/roles/${roleId}/permissions`, {
            body: JSON.stringify({ permission })
        })
      }
    },
    onSuccess: (_, vars) => {
      // Optimistic Update
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        if (vars.action === 'add') return [...old, vars.permission]
        return old.filter((p) => p !== vars.permission)
      })
      toast.success(vars.action === 'add' ? 'Permission assigned' : 'Permission revoked')
    },
    onError: () => toast.error('Failed to update permission'),
  })

  // Bulk Mutation
  const bulkMutation = useMutation({
    mutationFn: async ({ permissions, action }: { permissions: string[]; action: 'add' | 'remove' }) => {
      return apiClient.post(`/api/realms/${realm}/rbac/roles/${roleId}/permissions/bulk`, {
        permissions,
        action,
      })
    },
    onSuccess: (_, vars) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        if (vars.action === 'add') return Array.from(new Set([...old, ...vars.permissions]))
        return old.filter((p) => !vars.permissions.includes(p))
      })
      toast.success(vars.action === 'add' ? 'Resources assigned' : 'Resources cleared')
    },
    onError: () => toast.error('Failed to update permissions'),
  })

  return { toggleMutation, bulkMutation }
}
