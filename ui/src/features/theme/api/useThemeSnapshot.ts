import { useQuery } from '@tanstack/react-query'

import type { ThemeSnapshot } from '@/entities/theme/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

type ThemeSnapshotParams = {
  pageKey?: string
  nodeKey?: string
  clientId?: string
}

export function useThemeSnapshot(
  realm?: string,
  params?: ThemeSnapshotParams,
  options?: { enabled?: boolean },
) {
  const searchParams = new URLSearchParams()
  if (params?.pageKey) {
    searchParams.set('page_key', params.pageKey)
  }
  if (params?.nodeKey) {
    searchParams.set('node_key', params.nodeKey)
  }
  if (params?.clientId) {
    searchParams.set('client_id', params.clientId)
  }
  const query = searchParams.toString()

  return useQuery<ThemeSnapshot>({
    queryKey: queryKeys.themeSnapshot(realm ?? '', params),
    queryFn: () =>
      apiClient.get<ThemeSnapshot>(
        `/api/realms/${realm}/theme/resolve${query ? `?${query}` : ''}`,
      ),
    enabled: options?.enabled ?? !!realm,
  })
}
