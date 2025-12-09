import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import type { FlowDraft } from '@/entities/flow/model/types.ts'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useFlowDraft(draftId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['flow-draft', realm, draftId],
    queryFn: async () => {
      const draft = await apiClient.get<{
        graph_json: string | any
        [key: string]: any
      }>(`/api/realms/${realm}/flows/drafts/${draftId}`)

      return {
        ...draft,
        // Ensure graph_json is parsed if it's a string, but keep the FlowDraft type happy
        graph_json:
          typeof draft.graph_json === 'string' ? JSON.parse(draft.graph_json) : draft.graph_json,
      } as FlowDraft & { graph_json: any } // Intersection type to allow graph_json to be object
    },
    enabled: !!draftId && !!realm,
  })
}

export function useSaveDraft() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({
      draftId,
      graph,
      name,
    }: {
      draftId: string
      graph: any
      name?: string
    }) => {
      // Serialize back to string or let Axios handle JSON object (Backend expects string or JSON value)
      return apiClient.put(`/api/realms/${realm}/flows/drafts/${draftId}`, {
        graph_json: graph, // Axios sends as JSON Object, Backend deserializes to serde_json::Value -> String
        name,
      })
    },
    onSuccess: (_, { draftId }) => {
      void queryClient.invalidateQueries({ queryKey: ['flow-draft', realm, draftId] })
    },
  })
}
