import { useQuery } from '@tanstack/react-query'

import type { Theme } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useThemes() {
  const realm = useActiveRealm()

  return useQuery<Theme[]>({
    queryKey: ['themes', realm],
    queryFn: () => apiClient.get<Theme[]>(`/api/realms/${realm}/themes`),
    enabled: !!realm,
  })
}
