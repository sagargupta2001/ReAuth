import { useMutation } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'

type CleanupResponse = {
  deleted: number
}

export function useTelemetryClearLogs() {
  return useMutation({
    mutationFn: async (before?: string) => {
      const payload = before ? { before } : {}
      return apiClient.post<CleanupResponse>(
        '/api/system/observability/logs/clear',
        payload,
      )
    },
  })
}

export function useTelemetryClearTraces() {
  return useMutation({
    mutationFn: async (before?: string) => {
      const payload = before ? { before } : {}
      return apiClient.post<CleanupResponse>(
        '/api/system/observability/traces/clear',
        payload,
      )
    },
  })
}
