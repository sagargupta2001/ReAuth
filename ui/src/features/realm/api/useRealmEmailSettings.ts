import { useQuery } from '@tanstack/react-query'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { apiClient } from '@/shared/api/client.ts'

import type { RealmEmailSettings } from '@/entities/realm/model/types.ts'

export function useRealmEmailSettings() {
  const { data: realm } = useCurrentRealm()

  return useQuery({
    queryKey: ['realm-email-settings', realm?.id],
    queryFn: async () => {
      if (!realm?.id) {
        throw new Error('Realm not loaded')
      }
      return apiClient.get<RealmEmailSettings>(`/api/realms/${realm.id}/email-settings`)
    },
    enabled: !!realm?.id,
  })
}
