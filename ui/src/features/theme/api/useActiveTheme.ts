import { useQuery } from '@tanstack/react-query'

import type { ActiveThemeResponse } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useActiveTheme() {
  const realm = useActiveRealm()

  return useQuery<ActiveThemeResponse>({
    queryKey: queryKeys.activeTheme(realm),
    queryFn: () => apiClient.get<ActiveThemeResponse>(`/api/realms/${realm}/themes/active`),
    enabled: !!realm,
  })
}
