import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface ThemeDefaults {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
}

/**
 * The tokens and layout a freshly seeded theme starts from.
 *
 * Served by the backend so "reset to defaults" cannot drift from what seeding
 * actually produces — the UI used to keep its own copy, and the two diverged.
 */
export function useThemeDefaults() {
  const realm = useActiveRealm()

  return useQuery<ThemeDefaults>({
    queryKey: queryKeys.themeDefaults(realm),
    queryFn: () => apiClient.get<ThemeDefaults>(`/api/realms/${realm}/theme-defaults`),
    enabled: !!realm,
    // Compiled into the binary, so it cannot change while the app is open.
    staleTime: Infinity,
  })
}
