import { useQuery } from '@tanstack/react-query'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { apiClient } from '@/shared/api/client.ts'

import type { RealmRecoverySettings } from '@/entities/realm/model/types.ts'

export function useRealmRecoverySettings() {
  const { data: realm } = useCurrentRealm()

  return useQuery({
    queryKey: ['realm-recovery-settings', realm?.id],
    queryFn: async () => {
      if (!realm?.id) {
        throw new Error('Realm not loaded')
      }
      return apiClient.get<RealmRecoverySettings>(`/api/realms/${realm.id}/recovery-settings`)
    },
    enabled: !!realm?.id,
  })
}
