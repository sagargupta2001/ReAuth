import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface UserRoleRow {
  id: string
  name: string
  description?: string | null
  is_direct: boolean
  is_effective: boolean
}

export interface UserRoleListParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filter?: 'all' | 'direct' | 'effective' | 'unassigned'
}

export function useUserRolesList(userId: string, params: UserRoleListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['user-role-list', realm, userId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.filter) query.set('filter', params.filter)

      return apiClient.get<PaginatedResponse<UserRoleRow>>(
        `/api/realms/${realm}/users/${userId}/roles/list?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useUserRoleIds(userId: string, scope: 'direct' | 'effective' = 'direct') {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['user-roles', realm, userId, scope],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('scope', scope)
      return apiClient.get<string[]>(
        `/api/realms/${realm}/users/${userId}/roles?${query.toString()}`,
      )
    },
  })
}

export function useManageUserRoles(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const directQueryKey = ['user-roles', realm, userId, 'direct']
  const effectiveQueryKey = ['user-roles', realm, userId, 'effective']
  const listQueryKey = ['user-role-list', realm, userId]

  const addMutation = useMutation({
    mutationFn: async (roleId: string) => {
      return apiClient.post(`/api/realms/${realm}/users/${userId}/roles`, { role_id: roleId })
    },
    onSuccess: (_, roleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) => {
        if (old.includes(roleId)) return old
        return [...old, roleId]
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Role assigned to user')
    },
    onError: () => toast.error('Failed to assign role'),
  })

  const removeMutation = useMutation({
    mutationFn: async (roleId: string) => {
      return apiClient.delete(`/api/realms/${realm}/users/${userId}/roles/${roleId}`)
    },
    onSuccess: (_, roleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => id !== roleId),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Role removed from user')
    },
    onError: () => toast.error('Failed to remove role'),
  })

  const bulkAddMutation = useMutation({
    mutationFn: async (roleIds: string[]) => {
      await Promise.all(
        roleIds.map((roleId) =>
          apiClient.post(`/api/realms/${realm}/users/${userId}/roles`, { role_id: roleId }),
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
      toast.success('Roles assigned to user')
    },
    onError: () => toast.error('Failed to assign roles'),
  })

  const bulkRemoveMutation = useMutation({
    mutationFn: async (roleIds: string[]) => {
      await Promise.all(
        roleIds.map((roleId) =>
          apiClient.delete(`/api/realms/${realm}/users/${userId}/roles/${roleId}`),
        ),
      )
    },
    onSuccess: (_, roleIds) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => !roleIds.includes(id)),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Roles removed from user')
    },
    onError: () => toast.error('Failed to remove roles'),
  })

  return { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation }
}
