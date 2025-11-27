import { useQuery } from '@tanstack/react-query'

import type { OidcClient } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useClient(clientId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['client', realm, clientId],
    queryFn: () => apiClient.get<OidcClient>(`/api/realms/${realm}/clients/${clientId}`),
    enabled: !!clientId,
  })
}
