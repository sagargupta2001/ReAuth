import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface GroupMemberRow {
  id: string
  username: string
  is_member: boolean
}

export interface GroupMemberListParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  filter?: 'all' | 'members' | 'non-members'
}

export function useGroupMembersList(groupId: string, params: GroupMemberListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-member-list', realm, groupId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.filter) query.set('filter', params.filter)

      return apiClient.get<PaginatedResponse<GroupMemberRow>>(
        `/api/realms/${realm}/rbac/groups/${groupId}/members/list?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useGroupMemberIds(groupId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['group-members', realm, groupId],
    queryFn: async () => {
      return apiClient.get<string[]>(`/api/realms/${realm}/rbac/groups/${groupId}/members`)
    },
  })
}

export function useManageGroupMembers(groupId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const queryKey = ['group-members', realm, groupId]
  const listQueryKey = ['group-member-list', realm, groupId]

  const addMutation = useMutation({
    mutationFn: async (userId: string) => {
      return apiClient.post(`/api/realms/${realm}/rbac/groups/${groupId}/members`, {
        user_id: userId,
      })
    },
    onSuccess: (_, userId) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        if (old.includes(userId)) return old
        return [...old, userId]
      })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('User added to group')
    },
    onError: () => toast.error('Failed to add user to group'),
  })

  const removeMutation = useMutation({
    mutationFn: async (userId: string) => {
      return apiClient.delete(
        `/api/realms/${realm}/rbac/groups/${groupId}/members/${userId}`,
      )
    },
    onSuccess: (_, userId) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) =>
        old.filter((id) => id !== userId),
      )
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('User removed from group')
    },
    onError: () => toast.error('Failed to remove user from group'),
  })

  const bulkAddMutation = useMutation({
    mutationFn: async (userIds: string[]) => {
      await Promise.all(
        userIds.map((userId) =>
          apiClient.post(`/api/realms/${realm}/rbac/groups/${groupId}/members`, {
            user_id: userId,
          }),
        ),
      )
    },
    onSuccess: (_, userIds) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) => {
        const merged = new Set([...old, ...userIds])
        return Array.from(merged)
      })
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Users added to group')
    },
    onError: () => toast.error('Failed to add users to group'),
  })

  const bulkRemoveMutation = useMutation({
    mutationFn: async (userIds: string[]) => {
      await Promise.all(
        userIds.map((userId) =>
          apiClient.delete(`/api/realms/${realm}/rbac/groups/${groupId}/members/${userId}`),
        ),
      )
    },
    onSuccess: (_, userIds) => {
      queryClient.setQueryData(queryKey, (old: string[] = []) =>
        old.filter((id) => !userIds.includes(id)),
      )
      void queryClient.invalidateQueries({ queryKey: listQueryKey })
      toast.success('Users removed from group')
    },
    onError: () => toast.error('Failed to remove users from group'),
  })

  return { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation }
}
