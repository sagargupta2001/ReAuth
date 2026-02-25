import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface UpdateWebhookPayload {
  name?: string
  url?: string
  description?: string | null
  signing_secret?: string
  status?: string
  custom_headers?: Record<string, string>
}

export function useUpdateWebhook() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      endpointId,
      payload,
    }: {
      endpointId: string
      payload: UpdateWebhookPayload
    }) => apiClient.put(`/api/realms/${realm}/webhooks/${endpointId}`, payload),
    onSuccess: (_, vars) => {
      toast.success('Webhook updated')
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm] })
      void queryClient.invalidateQueries({ queryKey: ['webhooks', realm, vars.endpointId] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update webhook')
    },
  })
}
