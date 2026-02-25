import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { DeliveryLog } from '@/entities/events/model/types'
import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface DeliveryQueryParams {
  page?: number
  per_page?: number
  event_type?: string
  event_id?: string
  failed?: boolean
  start_time?: string
  end_time?: string
}

export function useWebhookDeliveries(endpointId?: string, params: DeliveryQueryParams = {}) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['webhook-deliveries', realm, endpointId, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page ?? 1))
      query.set('per_page', String(params.per_page ?? 50))
      if (params.event_type) query.set('event_type', params.event_type)
      if (params.event_id) query.set('event_id', params.event_id)
      if (params.failed !== undefined) query.set('failed', String(params.failed))
      if (params.start_time) query.set('start_time', params.start_time)
      if (params.end_time) query.set('end_time', params.end_time)

      return apiClient.get<PaginatedResponse<DeliveryLog>>(
        `/api/realms/${realm}/webhooks/${endpointId}/deliveries?${query.toString()}`,
      )
    },
    enabled: !!realm && !!endpointId,
    placeholderData: keepPreviousData,
  })
}
