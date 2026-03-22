import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client.ts'

import type { RealmRecoverySettings } from '@/entities/realm/model/types.ts'

type UpdateRealmRecoverySettingsPayload = {
  token_ttl_minutes?: number
  rate_limit_max?: number
  rate_limit_window_minutes?: number
  revoke_sessions_on_reset?: boolean
  email_subject?: string | null
  email_body?: string | null
}

export function useUpdateRealmRecoverySettings(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: UpdateRealmRecoverySettingsPayload) =>
      apiClient.put<RealmRecoverySettings>(`/api/realms/${realmId}/recovery-settings`, payload),
    onSuccess: (data) => {
      toast.success('Recovery settings updated successfully.')
      queryClient.setQueryData(['realm-recovery-settings', realmId], data)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
