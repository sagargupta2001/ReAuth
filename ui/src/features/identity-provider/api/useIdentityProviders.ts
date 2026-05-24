import { useQuery } from '@tanstack/react-query'

import type { IdentityProvider } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useIdentityProviders() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.identityProviders(realm),
    queryFn: () => apiClient.get<IdentityProvider[]>(`/api/realms/${realm}/identity-providers`),
  })
}
