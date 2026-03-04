import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export type ThemeBindingSummary = {
  client_id: string
  theme_id: string
  active_version_id: string
  active_version_number?: number | null
}

export function useThemeBindings(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeBindingSummary[]>({
    queryKey: ['theme-bindings', realm, themeId],
    queryFn: () =>
      apiClient.get<ThemeBindingSummary[]>(
        `/api/realms/${realm}/themes/${themeId}/bindings`,
      ),
    enabled: !!realm && !!themeId,
  })
}
