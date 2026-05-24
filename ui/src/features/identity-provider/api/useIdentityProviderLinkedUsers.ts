import { useQuery } from '@tanstack/react-query'

import type { IdentityProviderLinkedUser } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useIdentityProviderLinkedUsers(providerId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.identityProviderLinkedUsers(realm, providerId),
    queryFn: () =>
      apiClient.get<IdentityProviderLinkedUser[]>(
        `/api/realms/${realm}/identity-providers/${providerId}/linked-users`,
      ),
    enabled: Boolean(providerId),
  })
}
