import { useQuery } from '@tanstack/react-query'

import type { IdentityProvider } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useIdentityProvider(providerId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.identityProvider(realm, providerId),
    queryFn: () =>
      apiClient.get<IdentityProvider>(`/api/realms/${realm}/identity-providers/${providerId}`),
    enabled: !!providerId,
  })
}
