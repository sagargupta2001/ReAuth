import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { ThemeDetails, ThemeVersion } from '@/entities/theme/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useActivateThemeLatest() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (themeId: string) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')

      const details = await apiClient.get<ThemeDetails>(`/api/realms/${realm}/themes/${themeId}`)
      let versionId = details.active_version_id ?? undefined

      if (!versionId) {
        const versions = await apiClient.get<ThemeVersion[]>(
          `/api/realms/${realm}/themes/${themeId}/versions`,
        )
        const latest = versions
          .slice()
          .sort((a, b) => b.version_number - a.version_number)[0]
        versionId = latest?.id
      }

      if (!versionId) {
        throw new Error('No published versions found for this theme.')
      }

      await apiClient.post(
        `/api/realms/${realm}/themes/${themeId}/versions/${versionId}/activate`,
        {},
      )
      return { themeId, versionId }
    },
    onSuccess: ({ themeId }) => {
      toast.success('Theme activated')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themeVersions(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themePreview(realm, themeId) })
    },
    onError: (error: unknown) => {
      const message = error instanceof Error ? error.message : 'Failed to activate theme'
      toast.error(message)
    },
  })
}
