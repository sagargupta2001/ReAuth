import { useQuery } from '@tanstack/react-query'

import type { RealmPasskeyAnalytics } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useRealmPasskeyAnalytics(realmId?: string, windowHours = 24) {
  return useQuery({
    queryKey: queryKeys.realmPasskeyAnalytics(realmId, windowHours),
    queryFn: async () => {
      if (!realmId) {
        throw new Error('Realm not loaded')
      }
      const query = new URLSearchParams({
        window_hours: String(windowHours),
      })
      return apiClient.get<RealmPasskeyAnalytics>(
        `/api/realms/${realmId}/passkey-settings/analytics?${query.toString()}`,
      )
    },
    enabled: Boolean(realmId),
    staleTime: 30_000,
  })
}
