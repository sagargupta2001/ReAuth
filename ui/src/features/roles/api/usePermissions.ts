import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface PermissionDef {
  id: string
  name: string
  description: string
  custom_id?: string
}

export interface ResourceGroup {
  id: string
  label: string
  description: string
  permissions: PermissionDef[]
}

export function usePermissions(clientId?: string | null) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['permissions-definitions', realm, clientId ?? null],
    queryFn: async () => {
      const query = new URLSearchParams()
      if (clientId) query.set('client_id', clientId)
      const suffix = query.toString()
      return apiClient.get<ResourceGroup[]>(
        `/api/realms/${realm}/rbac/permissions${suffix ? `?${suffix}` : ''}`,
      )
    },
    // Definitions rarely change, so cache them for a long time (1 hour)
    staleTime: 1000 * 60 * 60,
  })
}
