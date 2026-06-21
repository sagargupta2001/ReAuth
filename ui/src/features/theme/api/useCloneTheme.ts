import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { ThemeDetails } from '@/entities/theme/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface CloneThemePayload {
  name: string
  make_active: boolean
}

export function useCloneTheme(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: CloneThemePayload) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return apiClient.post<ThemeDetails>(`/api/realms/${realm}/themes/${themeId}/clone`, payload)
    },
    onSuccess: () => {
      toast.success('Theme cloned.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.activeTheme(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to clone theme.')
    },
  })
}
