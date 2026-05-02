import { useQuery } from '@tanstack/react-query'

import type { RealmPasskeySettings } from '@/entities/realm/model/types.ts'
import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useRealmPasskeySettings() {
  const { data: realm } = useCurrentRealm()

  return useQuery({
    queryKey: queryKeys.realmPasskeySettings(realm?.id),
    queryFn: async () => {
      if (!realm?.id) {
        throw new Error('Realm not loaded')
      }
      return apiClient.get<RealmPasskeySettings>(`/api/realms/${realm.id}/passkey-settings`)
    },
    enabled: !!realm?.id,
  })
}
