import { useQuery } from '@tanstack/react-query'

import type { OidcClient } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export async function fetchClient(realm: string, clientId: string) {
  return apiClient.get<OidcClient>(`/api/realms/${realm}/clients/${clientId}`)
}

export function useClient(clientId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.client(realm, clientId),
    queryFn: () => fetchClient(realm, clientId),
    enabled: !!clientId,
  })
}
