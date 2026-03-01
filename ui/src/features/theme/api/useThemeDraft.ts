import { useQuery } from '@tanstack/react-query'

import type { ThemeDraft } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useThemeDraft(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeDraft>({
    queryKey: ['themes', realm, themeId, 'draft'],
    queryFn: () => apiClient.get<ThemeDraft>(`/api/realms/${realm}/themes/${themeId}/draft`),
    enabled: !!realm && !!themeId,
  })
}
