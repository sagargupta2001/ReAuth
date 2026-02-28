import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface EventRoutingMetrics {
  window_hours: number
  total_routed: number
  success_rate: number
  avg_latency_ms: number | null
}

export function useEventRoutingMetrics(windowHours = 24) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['event-routing-metrics', realm, windowHours],
    queryFn: async () =>
      apiClient.get<EventRoutingMetrics>(
        `/api/realms/${realm}/webhooks/metrics?window_hours=${windowHours}`,
      ),
    staleTime: 10_000,
    refetchInterval: 30_000,
  })
}
