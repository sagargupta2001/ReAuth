import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useDeleteWebhook() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (endpointId: string) =>
      apiClient.delete(`/api/realms/${realm}/webhooks/${endpointId}`),
    onSuccess: (_, endpointId) => {
      toast.success('Webhook deleted')
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm, endpointId] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to delete webhook')
    },
  })
}
