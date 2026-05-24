import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { RealmIdpSettings } from '@/entities/realm/model/types.ts'

type UpdateRealmIdpSettingsPayload = {
  oauth_start_rate_limit_max?: number
  oauth_start_rate_limit_window_minutes?: number
}

export function useUpdateRealmIdpSettings(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: UpdateRealmIdpSettingsPayload) =>
      apiClient.put<RealmIdpSettings>(`/api/realms/${realmId}/idp-settings`, payload),
    onSuccess: (data) => {
      toast.success('Identity brokering settings updated successfully.')
      queryClient.setQueryData(queryKeys.realmIdpSettings(realmId), data)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
