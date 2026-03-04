import { useQuery } from '@tanstack/react-query'

import type { ThemeVersion } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useThemeVersions(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeVersion[]>({
    queryKey: ['themes', realm, themeId, 'versions'],
    queryFn: () => apiClient.get<ThemeVersion[]>(`/api/realms/${realm}/themes/${themeId}/versions`),
    enabled: !!realm && !!themeId,
  })
}
