import { useQuery } from '@tanstack/react-query'

import type { ThemeDraft } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export type ThemeVersionSnapshotResponse = {
  version_id: string
  theme_id: string
  version_number: number
  snapshot: ThemeDraft
}

export function useThemeVersionSnapshot(themeId?: string, versionId?: string | null) {
  const realm = useActiveRealm()

  return useQuery<ThemeVersionSnapshotResponse>({
    queryKey: ['themes', realm, themeId, 'versions', versionId, 'snapshot'],
    queryFn: () =>
      apiClient.get<ThemeVersionSnapshotResponse>(
        `/api/realms/${realm}/themes/${themeId}/versions/${versionId}/snapshot`,
      ),
    enabled: !!realm && !!themeId && !!versionId,
  })
}
