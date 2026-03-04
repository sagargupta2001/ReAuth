import { useQuery } from '@tanstack/react-query'

import type { ThemeAsset } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useThemeAssets(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeAsset[]>({
    queryKey: ['themes', realm, themeId, 'assets'],
    queryFn: () => apiClient.get<ThemeAsset[]>(`/api/realms/${realm}/themes/${themeId}/assets`),
    enabled: !!realm && !!themeId,
  })
}
