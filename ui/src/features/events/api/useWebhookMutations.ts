import { useMutation, useQueryClient } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useWebhookMutations() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  const invalidateWebhooks = (endpointId?: string) => {
    void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
    if (endpointId) {
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm, endpointId] })
    }
  }

  const enableWebhook = useMutation({
    mutationFn: (endpointId: string) =>
      apiClient.post(`/api/realms/${realm}/webhooks/${endpointId}/enable`, {}),
    onSuccess: (_, endpointId) => invalidateWebhooks(endpointId),
  })

  const disableWebhook = useMutation({
    mutationFn: ({ endpointId, reason }: { endpointId: string; reason?: string }) =>
      apiClient.post(`/api/realms/${realm}/webhooks/${endpointId}/disable`, {
        reason,
      }),
    onSuccess: (_, vars) => invalidateWebhooks(vars.endpointId),
  })

  return {
    enableWebhook,
    disableWebhook,
  }
}
