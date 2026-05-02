import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { RealmPasskeySettings } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

type UpdateRealmPasskeySettingsPayload = {
  enabled?: boolean
  allow_password_fallback?: boolean
  discoverable_preferred?: boolean
  challenge_ttl_secs?: number
  reauth_max_age_secs?: number
}

export function useUpdateRealmPasskeySettings(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: UpdateRealmPasskeySettingsPayload) =>
      apiClient.put<RealmPasskeySettings>(`/api/realms/${realmId}/passkey-settings`, payload),
    onSuccess: (data) => {
      toast.success('Passkey settings updated successfully.')
      queryClient.setQueryData(queryKeys.realmPasskeySettings(realmId), data)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
