import { useQuery } from '@tanstack/react-query'

import type { ThemePageTemplate } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useThemePages(themeId?: string) {
  const realm = useActiveRealm()
  const params = themeId ? `?theme_id=${themeId}` : ''

  return useQuery<ThemePageTemplate[]>({
    queryKey: queryKeys.themePages(realm, themeId ?? ''),
    queryFn: () =>
      apiClient.get<ThemePageTemplate[]>(`/api/realms/${realm}/themes/pages${params}`),
    enabled: !!realm,
  })
}
