import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { WebhookEndpointDetails } from '@/entities/events/model/types'
import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface WebhookListParams {
  page?: number
  per_page?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  q?: string
}

export function useWebhooks(params: WebhookListParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['webhooks', realm, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.page) query.set('page', String(params.page))
      if (params.per_page) query.set('per_page', String(params.per_page))
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.q) query.set('q', params.q)
      const queryString = query.toString()
      const url = queryString
        ? `/api/realms/${realm}/webhooks?${queryString}`
        : `/api/realms/${realm}/webhooks`
      return apiClient.get<PaginatedResponse<WebhookEndpointDetails>>(url)
    },
    enabled: !!realm,
    placeholderData: keepPreviousData,
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
