import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function usePublishFlow() {
  const realm = useActiveRealm()
  const { flowId } = useParams()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!flowId || !realm) throw new Error('Missing Flow ID or Realm')

      // Call the new publish endpoint
      // Adjust path if your router is /realms/:realm/flows/:id/publish
      return await apiClient.post(`/api/realms/${realm}/flows/${flowId}/publish`, {})
    },
    onSuccess: () => {
      toast.success('Flow published successfully!')
      // Invalidate queries to refresh the "Active Version" status in Details page
      void queryClient.invalidateQueries({ queryKey: ['flow', flowId] })
      void queryClient.invalidateQueries({ queryKey: ['flows'] })
      void queryClient.invalidateQueries({ queryKey: ['flow-draft'] })
      void queryClient.invalidateQueries({ queryKey: ['flow-versions', flowId] })
    },
    onError: (error: any) => {
      // Show the validation error from the backend (e.g. "Dead end detected")
      const serverMessage =
        error.response?.data?.error || // Standard Axios
        error.body?.error || // Some fetch wrappers
        error.message || // Fallback to generic JS error
        'Unknown validation error'

      toast.error(`Publish Failed: ${serverMessage}`)
    },
  })
}
