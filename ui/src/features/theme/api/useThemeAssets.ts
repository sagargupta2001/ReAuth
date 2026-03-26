import { useQuery } from '@tanstack/react-query'

import type { ThemeAsset } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useThemeAssets(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeAsset[]>({
    queryKey: queryKeys.themeAssets(realm, themeId ?? ''),
    queryFn: () => apiClient.get<ThemeAsset[]>(`/api/realms/${realm}/themes/${themeId}/assets`),
    enabled: !!realm && !!themeId,
  })
}
