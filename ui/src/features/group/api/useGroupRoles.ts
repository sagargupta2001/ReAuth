import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface GroupRoleRow {
  id: string
  name: string
  description?: string | null
  is_direct: boolean
  is_effective: boolean
}

export interface GroupRoleListParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filter?: 'all' | 'direct' | 'effective' | 'unassigned'
}

export function useGroupRolesList(groupId: string, params: GroupRoleListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-role-list', realm, groupId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.filter) query.set('filter', params.filter)

      return apiClient.get<PaginatedResponse<GroupRoleRow>>(
        `/api/realms/${realm}/rbac/groups/${groupId}/roles/list?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useGroupRoleIds(groupId: string, scope: 'direct' | 'effective' = 'direct') {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-roles', realm, groupId, scope],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('scope', scope)
      return apiClient.get<string[]>(
        `/api/realms/${realm}/rbac/groups/${groupId}/roles?${query.toString()}`,
      )
    },
  })
}

export function useManageGroupRoles(groupId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const directQueryKey = ['group-roles', realm, groupId, 'direct']
  const effectiveQueryKey = ['group-roles', realm, groupId, 'effective']
  const listQueryKey = ['group-role-list', realm, groupId]

  const addMutation = useMutation({
    mutationFn: async (roleId: string) => {
      return apiClient.post(`/api/realms/${realm}/rbac/groups/${groupId}/roles`, {
        role_id: roleId,
      })
    },
    onSuccess: (_, roleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) => {
        if (old.includes(roleId)) return old
        return [...old, roleId]
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Role assigned to group')
    },
    onError: () => toast.error('Failed to assign role'),
  })

  const removeMutation = useMutation({
    mutationFn: async (roleId: string) => {
      return apiClient.delete(`/api/realms/${realm}/rbac/groups/${groupId}/roles/${roleId}`)
    },
    onSuccess: (_, roleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => id !== roleId),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Role removed from group')
    },
    onError: () => toast.error('Failed to remove role'),
  })

  const bulkAddMutation = useMutation({
    mutationFn: async (roleIds: string[]) => {
      await Promise.all(
        roleIds.map((roleId) =>
          apiClient.post(`/api/realms/${realm}/rbac/groups/${groupId}/roles`, {
            role_id: roleId,
          }),
        ),
      )
    },
    onSuccess: (_, roleIds) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) => {
        const merged = new Set([...old, ...roleIds])
        return Array.from(merged)
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Roles assigned to group')
    },
    onError: () => toast.error('Failed to assign roles'),
  })

  const bulkRemoveMutation = useMutation({
    mutationFn: async (roleIds: string[]) => {
      await Promise.all(
        roleIds.map((roleId) =>
          apiClient.delete(`/api/realms/${realm}/rbac/groups/${groupId}/roles/${roleId}`),
        ),
      )
    },
    onSuccess: (_, roleIds) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => !roleIds.includes(id)),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Roles removed from group')
    },
    onError: () => toast.error('Failed to remove roles'),
  })

  return { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation }
}
