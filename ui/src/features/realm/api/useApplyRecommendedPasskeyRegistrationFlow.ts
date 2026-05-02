import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { RealmPasskeySettings } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

type ApplyRecommendedPasskeyRegistrationFlowResponse = {
  settings: RealmPasskeySettings
  registration_flow_version_id: string
  registration_flow_version_number: number
}

export function useApplyRecommendedPasskeyRegistrationFlow(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () =>
      apiClient.post<ApplyRecommendedPasskeyRegistrationFlowResponse>(
        `/api/realms/${realmId}/passkey-settings/recommended-registration-flow`,
        {},
      ),
    onSuccess: (data) => {
      toast.success(
        `Applied recommended registration passkey enrollment flow (v${data.registration_flow_version_number}).`,
      )
      queryClient.setQueryData(queryKeys.realmPasskeySettings(realmId), data.settings)
      void queryClient.invalidateQueries({ queryKey: queryKeys.flows() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowDrafts() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowDraft() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowVersions() })
    },
    onError: (err) => {
      toast.error(`Failed to apply recommended registration flow: ${err.message}`)
    },
  })
}
