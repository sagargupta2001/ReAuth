import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type { UserPhoneNumber } from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUserPhoneNumbers(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.userPhoneNumbers(userId),
    queryFn: () =>
      apiClient.get<UserPhoneNumber[]>(`/api/realms/${realm}/users/${userId}/phone-numbers`),
    enabled: Boolean(userId),
  })
}

export function useAddUserPhoneNumber(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: {
      phone_number: string
      is_primary?: boolean
      is_verified?: boolean
    }) => apiClient.post(`/api/realms/${realm}/users/${userId}/phone-numbers`, payload),
    onSuccess: () => {
      toast.success('Phone number added.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userPhoneNumbers(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to add phone number.')
    },
  })
}

export function useRemoveUserPhoneNumber(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (phoneNumberId: string) =>
      apiClient.delete(`/api/realms/${realm}/users/${userId}/phone-numbers/${phoneNumberId}`),
    onSuccess: () => {
      toast.success('Phone number removed.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userPhoneNumbers(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to remove phone number.')
    },
  })
}

export function useSetPrimaryPhoneNumber(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (phoneNumberId: string) =>
      apiClient.put(`/api/realms/${realm}/users/${userId}/phone-numbers/${phoneNumberId}/primary`, {}),
    onSuccess: () => {
      toast.success('Primary phone number updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userPhoneNumbers(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to set primary phone number.')
    },
  })
}

export function useSetPhoneNumberVerified(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      phoneNumberId,
      is_verified,
    }: {
      phoneNumberId: string
      is_verified: boolean
    }) =>
      apiClient.patch(
        `/api/realms/${realm}/users/${userId}/phone-numbers/${phoneNumberId}/verified`,
        { is_verified },
      ),
    onSuccess: () => {
      toast.success('Phone verification status updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userPhoneNumbers(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update verification status.')
    },
  })
}
