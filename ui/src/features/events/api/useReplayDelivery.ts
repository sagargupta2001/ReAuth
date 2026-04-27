import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useReplayDelivery() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (deliveryId: string) =>
      apiClient.post(`/api/system/observability/deliveries/${deliveryId}/replay`, {}),
    onSuccess: () => {
      toast.success('Replay sent')
      void queryClient.invalidateQueries({ queryKey: queryKeys.webhookDeliveries() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.webhookDeliveryLogs() })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to replay delivery')
    },
  })
}
