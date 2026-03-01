import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ThemeVersion } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function usePublishTheme(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return await apiClient.post<ThemeVersion>(
        `/api/realms/${realm}/themes/${themeId}/publish`,
        {},
      )
    },
    onSuccess: () => {
      toast.success('Theme published')
      void queryClient.invalidateQueries({ queryKey: ['themes', realm, themeId, 'versions'] })
      void queryClient.invalidateQueries({ queryKey: ['themes', realm, themeId] })
      void queryClient.invalidateQueries({ queryKey: ['themes', realm] })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to publish theme'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
