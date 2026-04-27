import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useParams } from 'react-router-dom'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'

export function usePublishFlow() {
  const realm = useActiveRealm()
  const { flowId } = useParams()
  const queryClient = useQueryClient()
  const setPublishError = useFlowBuilderStore((state) => state.setPublishError)

  return useMutation({
    mutationFn: async () => {
      setPublishError(null)
      if (!flowId || !realm) throw new Error('Missing Flow ID or Realm')

      // Call the new publish endpoint
      // Adjust path if your router is /realms/:realm/flows/:id/publish
      return await apiClient.post(`/api/realms/${realm}/flows/${flowId}/publish`, {})
    },
    onSuccess: () => {
      toast.success('Flow published successfully!')
      setPublishError(null)
      // 1. REFRESH SIDEBAR: Invalidate the specific binding hook
      void queryClient.invalidateQueries({ queryKey: queryKeys.realmBindings() })

      // 2. Refresh the Realm config itself (good practice)
      void queryClient.invalidateQueries({ queryKey: queryKeys.realm(realm) })

      // Invalidate queries to refresh the "Active Version" status in Details page
      void queryClient.invalidateQueries({ queryKey: queryKeys.flow(flowId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flows() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowDraft() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.flowVersions(flowId) })
    },
    onError: (error: unknown) => {
      // Show the validation error from the backend (e.g. "Dead end detected")
      let serverMessage = 'Unknown validation error'
      let issues: Array<{ message: string; node_ids: string[] }> = []

      if (error && typeof error === 'object') {
        const errObj = error as Record<string, unknown>
        const response = errObj.response as Record<string, unknown> | undefined
        const responseData = response?.data as Record<string, unknown> | undefined
        const responseCode = responseData?.code as string | undefined
        const details = responseData?.details as
          | { message?: string; issues?: Array<{ message?: string; node_ids?: string[] }> }
          | undefined
        const body = errObj.body as Record<string, unknown> | undefined

        serverMessage =
          (responseData?.error as string) ||
          (body?.error as string) ||
          (errObj.message as string) ||
          serverMessage

        if (details?.issues) {
          issues = details.issues
            .filter((issue) => issue && typeof issue.message === 'string')
            .map((issue) => ({
              message: issue.message ?? 'Validation failed',
              node_ids: Array.isArray(issue.node_ids) ? issue.node_ids : [],
            }))
        }

        if (responseCode === 'validation.failed') {
          const prefix = 'Validation failed: '
          if (serverMessage.startsWith(prefix)) {
            serverMessage = serverMessage.slice(prefix.length)
          }
        }
      }

      setPublishError({ message: serverMessage, issues })
      toast.error(`Publish Failed: ${serverMessage}`)
    },
  })
}
