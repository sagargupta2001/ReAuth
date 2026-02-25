import { useQuery } from '@tanstack/react-query'

import type { WebhookEndpointDetails } from '@/entities/events/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useWebhooks() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['webhooks', realm],
    queryFn: () => apiClient.get<WebhookEndpointDetails[]>(`/api/realms/${realm}/webhooks`),
    enabled: !!realm,
  })
}

export function useWebhook(endpointId?: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['webhooks', realm, endpointId],
    queryFn: () =>
      apiClient.get<WebhookEndpointDetails>(`/api/realms/${realm}/webhooks/${endpointId}`),
    enabled: !!realm && !!endpointId,
  })
}
