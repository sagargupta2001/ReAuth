import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

type ThemeTemplateGapResponse = {
  missing: string[]
}

export function useThemeTemplateGaps(themeId?: string) {
  const realm = useActiveRealm()

  return useQuery<ThemeTemplateGapResponse>({
    queryKey: ['theme-template-gaps', realm, themeId],
    queryFn: () =>
      apiClient.get<ThemeTemplateGapResponse>(
        `/api/realms/${realm}/themes/${themeId}/template-gaps`,
      ),
    enabled: !!realm && !!themeId,
  })
}
