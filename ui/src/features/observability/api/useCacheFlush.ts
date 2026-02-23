import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client'

export interface CacheFlushResponse {
  flushed: string
}

export function useCacheFlush() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (namespace?: string) => {
      return apiClient.post<CacheFlushResponse>(
        '/api/system/observability/cache/flush',
        namespace ? { namespace } : { namespace: 'all' },
      )
    },
    onSuccess: (data) => {
      void queryClient.invalidateQueries({ queryKey: ['observability-cache-stats'] })
      toast.success(`Cache flushed: ${data.flushed}`)
    },
    onError: () => toast.error('Failed to flush cache'),
  })
}
