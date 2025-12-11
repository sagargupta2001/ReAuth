import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useRollbackFlow() {
  const realm = useActiveRealm()
  const { flowId } = useParams()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (versionNumber: number) => {
      if (!flowId || !realm) throw new Error('Missing ID')
      return await apiClient.post(`/api/realms/${realm}/flows/${flowId}/rollback`, {
        version_number: versionNumber,
      })
    },
    onSuccess: () => {
      toast.success('Flow rolled back successfully')
      // Refresh details to show new active version
      void queryClient.invalidateQueries({ queryKey: ['flow', flowId] })
      // Refresh history list to update the "Active" badge
      void queryClient.invalidateQueries({ queryKey: ['flow-versions', flowId] })
      void queryClient.invalidateQueries({ queryKey: ['flow-draft'] })
    },
    onError: (err) => {
      toast.error('Rollback failed')
      console.error(err)
    },
  })
}
