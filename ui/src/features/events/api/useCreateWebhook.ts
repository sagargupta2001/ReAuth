import { useMutation, useQueryClient } from '@tanstack/react-query'

import type { CreateWebhookPayload, WebhookEndpointDetails } from '@/entities/events/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useCreateWebhook() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: CreateWebhookPayload) =>
      apiClient.post<WebhookEndpointDetails>(`/api/realms/${realm}/webhooks`, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
    },
  })
}
