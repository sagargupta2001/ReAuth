import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface RoleCompositeRow {
  id: string
  name: string
  description?: string | null
  is_direct: boolean
  is_effective: boolean
}

export interface RoleCompositeListParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filter?: 'all' | 'direct' | 'effective' | 'unassigned'
}

export function useRoleCompositesList(roleId: string, params: RoleCompositeListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['role-composite-list', realm, roleId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.filter) query.set('filter', params.filter)

      return apiClient.get<PaginatedResponse<RoleCompositeRow>>(
        `/api/realms/${realm}/rbac/roles/${roleId}/composites/list?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useRoleCompositeIds(roleId: string, scope: 'direct' | 'effective' = 'direct') {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['role-composites', realm, roleId, scope],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('scope', scope)
      return apiClient.get<string[]>(
        `/api/realms/${realm}/rbac/roles/${roleId}/composites?${query.toString()}`,
      )
    },
  })
}

export function useManageRoleComposites(roleId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const directQueryKey = ['role-composites', realm, roleId, 'direct']
  const effectiveQueryKey = ['role-composites', realm, roleId, 'effective']
  const listQueryKey = ['role-composite-list', realm, roleId]

  const addMutation = useMutation({
    mutationFn: async (childRoleId: string) => {
      return apiClient.post(`/api/realms/${realm}/rbac/roles/${roleId}/composites`, {
        role_id: childRoleId,
      })
    },
    onSuccess: (_, childRoleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) => {
        if (old.includes(childRoleId)) return old
        return [...old, childRoleId]
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Composite role added')
    },
    onError: () => toast.error('Failed to add composite role'),
  })

  const removeMutation = useMutation({
    mutationFn: async (childRoleId: string) => {
      return apiClient.delete(
        `/api/realms/${realm}/rbac/roles/${roleId}/composites/${childRoleId}`,
      )
    },
    onSuccess: (_, childRoleId) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => id !== childRoleId),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Composite role removed')
    },
    onError: () => toast.error('Failed to remove composite role'),
  })

  const bulkAddMutation = useMutation({
    mutationFn: async (childRoleIds: string[]) => {
      await Promise.all(
        childRoleIds.map((childRoleId) =>
          apiClient.post(`/api/realms/${realm}/rbac/roles/${roleId}/composites`, {
            role_id: childRoleId,
          }),
        ),
      )
    },
    onSuccess: (_, childRoleIds) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) => {
        const merged = new Set([...old, ...childRoleIds])
        return Array.from(merged)
      })
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Composite roles added')
    },
    onError: () => toast.error('Failed to add composite roles'),
  })

  const bulkRemoveMutation = useMutation({
    mutationFn: async (childRoleIds: string[]) => {
      await Promise.all(
        childRoleIds.map((childRoleId) =>
          apiClient.delete(
            `/api/realms/${realm}/rbac/roles/${roleId}/composites/${childRoleId}`,
          ),
        ),
      )
    },
    onSuccess: (_, childRoleIds) => {
      queryClient.setQueryData(directQueryKey, (old: string[] = []) =>
        old.filter((id) => !childRoleIds.includes(id)),
      )
      void queryClient.invalidateQueries({ queryKey: effectiveQueryKey })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Composite roles removed')
    },
    onError: () => toast.error('Failed to remove composite roles'),
  })

  return { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation }
}
