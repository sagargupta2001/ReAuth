import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface WebhookSubscriptionToggle {
  event_type: string
  enabled: boolean
}

export function useUpdateWebhookSubscriptions() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      endpointId,
      subscriptions,
    }: {
      endpointId: string
      subscriptions: WebhookSubscriptionToggle[]
    }) =>
      apiClient.post(`/api/realms/${realm}/webhooks/${endpointId}/subscriptions`, {
        subscriptions,
      }),
    onSuccess: (_, vars) => {
      toast.success('Webhook subscriptions updated')
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm, vars.endpointId] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update webhook subscriptions')
    },
  })
}
