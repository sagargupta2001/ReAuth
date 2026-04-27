import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useActivateThemeVersion(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (versionId: string) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return await apiClient.post(
        `/api/realms/${realm}/themes/${themeId}/versions/${versionId}/activate`,
        {},
      )
    },
    onSuccess: () => {
      toast.success('Theme version activated')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themeVersions(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themePreview(realm, themeId) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to activate version'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
