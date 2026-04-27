import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUpsertThemeBinding(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ clientId, versionId }: { clientId: string; versionId: string }) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return apiClient.put(`/api/realms/${realm}/themes/${themeId}/bindings/${clientId}`, {
        version_id: versionId,
      })
    },
    onSuccess: (_data, variables) => {
      toast.success('Client override saved')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themeBindings(realm, themeId) })
      void queryClient.invalidateQueries({
        queryKey: queryKeys.themeBindingClient(realm, variables.clientId),
      })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themePreview(realm, themeId) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to save client override'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
