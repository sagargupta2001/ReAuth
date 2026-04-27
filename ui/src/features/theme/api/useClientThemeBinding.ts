import { useQuery } from '@tanstack/react-query'

import type { ThemeBindingSummary } from '@/features/theme/api/useThemeBindings'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useClientThemeBinding(clientId?: string | null) {
  const realm = useActiveRealm()

  return useQuery<ThemeBindingSummary | null>({
    queryKey: queryKeys.themeBindingClient(realm, clientId ?? ''),
    queryFn: () =>
      apiClient.get<ThemeBindingSummary | null>(
        `/api/realms/${realm}/themes/client-bindings/${clientId}`,
      ),
    enabled: !!realm && !!clientId,
  })
}
