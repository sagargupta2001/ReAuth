import { useQuery } from '@tanstack/react-query'

import type { ThemeSnapshot } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

type ThemePreviewParams = {
  pageKey?: string
  nodeKey?: string
}

export function useThemePreview(themeId?: string, params?: ThemePreviewParams) {
  const realm = useActiveRealm()
  const searchParams = new URLSearchParams()
  if (params?.pageKey) {
    searchParams.set('page_key', params.pageKey)
  }
  if (params?.nodeKey) {
    searchParams.set('node_key', params.nodeKey)
  }
  const query = searchParams.toString()

  return useQuery<ThemeSnapshot>({
    queryKey: ['theme-preview', realm, themeId, params?.pageKey, params?.nodeKey],
    queryFn: () =>
      apiClient.get<ThemeSnapshot>(
        `/api/realms/${realm}/themes/${themeId}/preview${query ? `?${query}` : ''}`,
      ),
    enabled: !!realm && !!themeId,
  })
}
