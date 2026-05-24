import { useQuery } from '@tanstack/react-query'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { RealmIdpSettings } from '@/entities/realm/model/types.ts'

export function useRealmIdpSettings() {
  const { data: realm } = useCurrentRealm()

  return useQuery({
    queryKey: queryKeys.realmIdpSettings(realm?.id),
    queryFn: async () => {
      if (!realm?.id) {
        throw new Error('Realm not loaded')
      }
      return apiClient.get<RealmIdpSettings>(`/api/realms/${realm.id}/idp-settings`)
    },
    enabled: !!realm?.id,
  })
}
