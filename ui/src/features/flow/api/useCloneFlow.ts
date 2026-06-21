import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { FlowDraft } from '@/entities/flow/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface CloneFlowPayload {
  name: string
  make_active: boolean
}

export function useCloneFlow(flowId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: CloneFlowPayload) => {
      if (!realm || !flowId) throw new Error('Missing Realm or Flow ID')
      return apiClient.post<FlowDraft>(`/api/realms/${realm}/flows/${flowId}/clone`, payload)
    },
    onSuccess: () => {
      toast.success('Flow cloned.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.flows(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to clone flow.')
    },
  })
}
