import { useQuery } from '@tanstack/react-query'

import type { WebhookEventCatalog } from '@/entities/events/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useWebhookEventCatalog() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.webhookEventCatalog(realm),
    queryFn: () => apiClient.get<WebhookEventCatalog>(`/api/realms/${realm}/webhooks/events`),
    enabled: !!realm,
    staleTime: 5 * 60 * 1000,
  })
}
