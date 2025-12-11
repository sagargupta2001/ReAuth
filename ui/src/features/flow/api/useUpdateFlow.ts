import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { FlowSettingsSchema } from '@/features/flow/model/settings-schema.ts'
import { apiClient } from '@/shared/api/client'

export function useUpdateFlow(flowId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (values: FlowSettingsSchema) => {
      if (!realm || !flowId) throw new Error('Missing Realm or Flow ID')

      // We use the existing update_draft endpoint
      return await apiClient.put(`/api/realms/${realm}/flows/drafts/${flowId}`, {
        name: values.name,
        description: values.description,
        // We don't send graph_json here, so the backend should preserve the existing one
        // (Ensure your backend update_draft_handler treats None/Null fields as "don't update")
      })
    },
    onSuccess: () => {
      toast.success('Flow settings updated')
      // Refresh the flow details
      void queryClient.invalidateQueries({ queryKey: ['flow-draft', realm, flowId] })
      // Refresh the sidebar list (since name changed)
      void queryClient.invalidateQueries({ queryKey: ['flows', realm] })
    },
    onError: (error: any) => {
      const msg = error.response?.data?.error || 'Failed to update flow'
      toast.error(msg)
    },
  })
}
