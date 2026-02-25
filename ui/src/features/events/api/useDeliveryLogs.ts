import { keepPreviousData, useQuery } from '@tanstack/react-query'

import type { DeliveryLog } from '@/entities/events/model/types'
import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { apiClient } from '@/shared/api/client'

export interface SystemDeliveryQueryParams {
  page?: number
  per_page?: number
  realm_id?: string
  target_type?: string
  target_id?: string
  event_type?: string
  event_id?: string
  failed?: boolean
  start?: string
  end?: string
  limit?: number
}

export function useDeliveryLogs(params: SystemDeliveryQueryParams = {}, enabled = true) {
  return useQuery({
    queryKey: ['delivery-logs', params],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (params.page) query.set('page', String(params.page))
      if (params.per_page) query.set('per_page', String(params.per_page))
      if (params.realm_id) query.set('realm_id', params.realm_id)
      if (params.target_type) query.set('target_type', params.target_type)
      if (params.target_id) query.set('target_id', params.target_id)
      if (params.event_type) query.set('event_type', params.event_type)
      if (params.event_id) query.set('event_id', params.event_id)
      if (params.failed !== undefined) query.set('failed', String(params.failed))
      if (params.start) query.set('start', params.start)
      if (params.end) query.set('end', params.end)
      if (params.limit) query.set('limit', String(params.limit))

      const suffix = query.toString()
      return apiClient.get<PaginatedResponse<DeliveryLog>>(
        `/api/system/observability/deliveries${suffix ? `?${suffix}` : ''}`,
      )
    },
    enabled,
    placeholderData: keepPreviousData,
  })
}
