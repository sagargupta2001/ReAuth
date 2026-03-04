import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ThemeDraft } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useSaveThemeDraft(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (draft: ThemeDraft) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return await apiClient.put(`/api/realms/${realm}/themes/${themeId}/draft`, draft)
    },
    onSuccess: () => {
      toast.success('Theme draft saved')
      void queryClient.invalidateQueries({ queryKey: ['themes', realm, themeId, 'draft'] })
      void queryClient.invalidateQueries({ queryKey: ['theme-preview', realm, themeId] })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to save theme draft'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
