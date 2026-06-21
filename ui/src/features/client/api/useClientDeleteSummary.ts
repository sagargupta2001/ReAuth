import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface ClientDeleteSummary {
  client_id: string
  name: string
  role_count: number
  permission_count: number
}

export function useClientDeleteSummary(clientId: string, enabled = false) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.clientDeleteSummary(realm, clientId),
    queryFn: () =>
      apiClient.get<ClientDeleteSummary>(
        `/api/realms/${realm}/clients/${clientId}/delete-summary`,
      ),
    enabled: enabled && !!clientId,
  })
}
