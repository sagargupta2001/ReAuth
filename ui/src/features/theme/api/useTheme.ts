import { useQuery } from '@tanstack/react-query'

import type { ThemeDetails } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useTheme(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeDetails>({
    queryKey: ['themes', realm, themeId],
    queryFn: () => apiClient.get<ThemeDetails>(`/api/realms/${realm}/themes/${themeId}`),
    enabled: !!realm && !!themeId,
  })
}
