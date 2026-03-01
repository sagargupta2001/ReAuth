import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ThemeDraft } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useStartThemeDraftFromVersion(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (versionId: string) => {
      return apiClient.post<ThemeDraft>(
        `/api/realms/${realm}/themes/${themeId}/versions/${versionId}/draft`,
        {},
      )
    },
    onSuccess: () => {
      toast.success('Draft created from version')
      void queryClient.invalidateQueries({ queryKey: ['theme-draft', realm, themeId] })
      void queryClient.invalidateQueries({ queryKey: ['theme-pages', realm, themeId] })
    },
    onError: () => {
      toast.error('Failed to start draft from version')
    },
  })
}
