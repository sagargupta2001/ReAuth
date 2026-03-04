import { useQuery } from '@tanstack/react-query'

import type { ThemeBindingSummary } from '@/features/theme/api/useThemeBindings'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useClientThemeBinding(clientId?: string | null) {
  const realm = useActiveRealm()

  return useQuery<ThemeBindingSummary | null>({
    queryKey: ['theme-bindings', 'client', realm, clientId],
    queryFn: () =>
      apiClient.get<ThemeBindingSummary | null>(
        `/api/realms/${realm}/themes/client-bindings/${clientId}`,
      ),
    enabled: !!realm && !!clientId,
  })
}
