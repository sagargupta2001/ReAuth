import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUpdateThemeFlowBinding(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (flowBindingId: string | null) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return await apiClient.put(`/api/realms/${realm}/themes/${themeId}`, {
        flow_binding_id: flowBindingId,
      })
    },
    onSuccess: () => {
      toast.success('Theme flow binding updated')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themePreview(realm, themeId) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to update theme flow binding'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
