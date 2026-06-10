import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { UserEmail } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUserEmails(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.userEmails(userId),
    queryFn: () => apiClient.get<UserEmail[]>(`/api/realms/${realm}/users/${userId}/emails`),
    enabled: Boolean(userId),
  })
}

export function useAddUserEmail(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: { email: string; is_primary?: boolean; is_verified?: boolean }) =>
      apiClient.post(`/api/realms/${realm}/users/${userId}/emails`, payload),
    onSuccess: () => {
      toast.success('Email address added.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userEmails(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to add email address.')
    },
  })
}

export function useRemoveUserEmail(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (emailId: string) =>
      apiClient.delete(`/api/realms/${realm}/users/${userId}/emails/${emailId}`),
    onSuccess: () => {
      toast.success('Email address removed.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userEmails(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to remove email address.')
    },
  })
}

export function useSetPrimaryEmail(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (emailId: string) =>
      apiClient.put(`/api/realms/${realm}/users/${userId}/emails/${emailId}/primary`, {}),
    onSuccess: () => {
      toast.success('Primary email updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userEmails(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to set primary email.')
    },
  })
}

export function useSetEmailVerified(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ emailId, is_verified }: { emailId: string; is_verified: boolean }) =>
      apiClient.patch(`/api/realms/${realm}/users/${userId}/emails/${emailId}/verified`, {
        is_verified,
      }),
    onSuccess: () => {
      toast.success('Email verification status updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userEmails(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update verification status.')
    },
  })
}
