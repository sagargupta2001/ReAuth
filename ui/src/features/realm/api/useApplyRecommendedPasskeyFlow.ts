import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { RealmPasskeySettings } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

type ApplyRecommendedPasskeyFlowResponse = {
  settings: RealmPasskeySettings
  browser_flow_version_id: string
  browser_flow_version_number: number
}

export function useApplyRecommendedPasskeyFlow(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () =>
      apiClient.post<ApplyRecommendedPasskeyFlowResponse>(
        `/api/realms/${realmId}/passkey-settings/recommended-browser-flow`,
        { enable_passkeys: true },
      ),
    onSuccess: (data) => {
      toast.success(
        `Applied recommended passkey-first browser flow (v${data.browser_flow_version_number}).`,
      )
      queryClient.setQueryData(queryKeys.realmPasskeySettings(realmId), data.settings)
      void queryClient.invalidateQueries({ queryKey: queryKeys.flows() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowDrafts() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowDraft() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowVersions() })
    },
    onError: (err) => {
      toast.error(`Failed to apply recommended flow: ${err.message}`)
    },
  })
}
