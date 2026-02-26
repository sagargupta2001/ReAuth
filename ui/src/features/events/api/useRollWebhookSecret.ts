import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useRollWebhookSecret() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (endpointId: string) =>
      apiClient.post(`/api/realms/${realm}/webhooks/${endpointId}/roll-secret`, {}),
    onSuccess: (_, endpointId) => {
      toast.success('Signing secret rolled')
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm, endpointId] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to roll signing secret')
    },
  })
}
