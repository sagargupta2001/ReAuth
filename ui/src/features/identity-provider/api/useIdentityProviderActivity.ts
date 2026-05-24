import { useQuery } from '@tanstack/react-query'

import type { IdentityProviderActivityFeed } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useIdentityProviderActivity(providerId: string, limit = 20) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.identityProviderActivity(realm, providerId, limit),
    queryFn: async () =>
      apiClient.get<IdentityProviderActivityFeed>(
        `/api/realms/${realm}/identity-providers/${providerId}/activity?limit=${limit}`,
      ),
    enabled: Boolean(providerId),
    refetchInterval: 30_000,
  })
}
