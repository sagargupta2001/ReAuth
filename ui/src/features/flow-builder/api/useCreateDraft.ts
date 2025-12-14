import { useMutation, useQueryClient } from '@tanstack/react-query'

import type { FlowDraft } from '@/entities/flow/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

interface CreateDraftPayload {
  name: string
  description?: string
}

export function useCreateDraft() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: CreateDraftPayload) =>
      apiClient.post<FlowDraft>(`/api/realms/${realm}/flows/drafts`, payload),
    onSuccess: () => {
      // Refresh the sidebar list
      void queryClient.invalidateQueries({ queryKey: ['flows', realm] })
    },
  })
}
