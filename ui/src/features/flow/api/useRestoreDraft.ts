import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useRestoreDraft() {
  const realm = useActiveRealm()
  const { flowId } = useParams()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (versionNumber: number) => {
      if (!flowId || !realm) throw new Error('Missing ID')
      return await apiClient.post(`/api/realms/${realm}/flows/${flowId}/restore-draft`, {
        version_number: versionNumber,
      })
    },
    onSuccess: () => {
      toast.success('Draft restored from history')
      // Refresh the visual overview immediately
      void queryClient.invalidateQueries({ queryKey: ['flow-draft'] })
    },
    onError: (err) => {
      toast.error('Failed to restore draft')
      console.error(err)
    },
  })
}
