import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface PermissionDef {
  id: string
  name: string
  description: string
}

export interface ResourceGroup {
  id: string
  label: string
  description: string
  permissions: PermissionDef[]
}

export function usePermissions() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['permissions-definitions', realm],
    queryFn: async () => {
      return apiClient.get<ResourceGroup[]>(`/api/realms/${realm}/rbac/permissions`)
    },
    // Definitions rarely change, so cache them for a long time (1 hour)
    staleTime: 1000 * 60 * 60,
  })
}
