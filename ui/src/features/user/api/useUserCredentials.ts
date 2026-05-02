import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface UserPasskeyCredential {
  id: string
  credential_id_b64url: string
  friendly_name?: string | null
  backed_up: boolean
  backup_eligible: boolean
  sign_count: number
  created_at: string
  last_used_at?: string | null
}

export interface UserCredentials {
  user_id: string
  password: {
    configured: boolean
    force_reset_on_next_login: boolean
    password_login_disabled: boolean
  }
  passkeys: UserPasskeyCredential[]
}

export function useUserCredentials(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.userCredentials(userId),
    queryFn: () => apiClient.get<UserCredentials>(`/api/realms/${realm}/users/${userId}/credentials`),
    enabled: Boolean(userId),
  })
}

export function useUpdateUserPassword(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (password: string) => {
      return apiClient.put(`/api/realms/${realm}/users/${userId}/credentials/password`, {
        password,
      })
    },
    onSuccess: () => {
      toast.success('Password updated successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userCredentials(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update password.')
    },
  })
}

export function useRevokeUserPasskey(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (credentialId: string) => {
      return apiClient.delete(`/api/realms/${realm}/users/${userId}/credentials/passkeys/${credentialId}`)
    },
    onSuccess: () => {
      toast.success('Passkey revoked successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userCredentials(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to revoke passkey.')
    },
  })
}

export function useRenameUserPasskey(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: { credentialId: string; friendlyName?: string | null }) => {
      return apiClient.put(
        `/api/realms/${realm}/users/${userId}/credentials/passkeys/${payload.credentialId}`,
        { friendly_name: payload.friendlyName ?? null },
      )
    },
    onSuccess: () => {
      toast.success('Passkey metadata updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userCredentials(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update passkey metadata.')
    },
  })
}

export function useUpdateUserPasswordPolicy(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: {
      force_reset_on_next_login?: boolean
      password_login_disabled?: boolean
    }) => {
      return apiClient.put(`/api/realms/${realm}/users/${userId}/credentials/password-policy`, payload)
    },
    onSuccess: () => {
      toast.success('Password policy updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userCredentials(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update password policy.')
    },
  })
}
