import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface RoleMemberRow {
  id: string
  username: string
  is_direct: boolean
  is_effective: boolean
}

export interface RoleMemberListParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filter?: 'all' | 'direct' | 'effective' | 'unassigned'
}

export function useRoleMembersList(roleId: string, params: RoleMemberListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['role-member-list', realm, roleId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.filter) query.set('filter', params.filter)

      return apiClient.get<PaginatedResponse<RoleMemberRow>>(
        `/api/realms/${realm}/rbac/roles/${roleId}/members/list?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useRoleMemberIds(roleId: string, scope: 'direct' | 'effective' = 'direct') {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['role-members', realm, roleId, scope],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('scope', scope)
      return apiClient.get<string[]>(
        `/api/realms/${realm}/rbac/roles/${roleId}/members?${query.toString()}`,
      )
    },
  })
}

export function useManageRoleMembers(roleId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const queryKey = ['role-members', realm, roleId, 'direct']
  const effectiveQueryKey = ['role-members', realm, roleId, 'effective']
  const listQueryKey = ['role-member-list', realm, roleId]

  const addMutation = useMutation({
    mutationFn: async (userId: string) => {
      return apiClient.post(`/api/realms/${realm}/users/${userId}/roles`, { role_id: roleId })
    },
    onSuccess: (_, userId) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        if (old.includes(userId)) return old
        return [...old, userId]
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('User assigned to role')
    },
    onError: () => toast.error('Failed to assign role'),
  })

  const removeMutation = useMutation({
    mutationFn: async (userId: string) => {
      return apiClient.delete(`/api/realms/${realm}/users/${userId}/roles/${roleId}`)
    },
    onSuccess: (_, userId) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) =>
        old.filter((id) => id !== userId),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('User removed from role')
    },
    onError: () => toast.error('Failed to remove role'),
  })

  const bulkAddMutation = useMutation({
    mutationFn: async (userIds: string[]) => {
      await Promise.all(
        userIds.map((userId) =>
          apiClient.post(`/api/realms/${realm}/users/${userId}/roles`, { role_id: roleId }),
        ),
      )
    },
    onSuccess: (_, userIds) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        const merged = new Set([...old, ...userIds])
        return Array.from(merged)
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Users assigned to role')
    },
    onError: () => toast.error('Failed to assign users'),
  })

  const bulkRemoveMutation = useMutation({
    mutationFn: async (userIds: string[]) => {
      await Promise.all(
        userIds.map((userId) =>
          apiClient.delete(`/api/realms/${realm}/users/${userId}/roles/${roleId}`),
        ),
      )
    },
    onSuccess: (_, userIds) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) =>
        old.filter((id) => !userIds.includes(id)),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Users removed from role')
    },
    onError: () => toast.error('Failed to remove users'),
  })

  return { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation }
}
