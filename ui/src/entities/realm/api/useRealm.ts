import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

import type { Realm } from '../model/types'

export function useCurrentRealm() {
  const realmName = useActiveRealm()

  return useQuery({
    // Include realmName in key so it refetches when you switch realms
    queryKey: ['realm', realmName],
    queryFn: async () => {
      // MVP: Fetch all and filter.
      // Production: You should eventually make an endpoint like GET /api/realms/:name
      const all = await apiClient.get<Realm[]>('/api/realms')
      const found = all.find((r) => r.name === realmName)

      if (!found) throw new Error(`Realm '${realmName}' not found`)

      return found
    },
    // Only fetch if we have a realm name
    enabled: !!realmName,
  })
}
