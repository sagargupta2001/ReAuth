// src/features/flow/api/useFlows.ts
import { useQuery } from '@tanstack/react-query'

import type { UnifiedFlowDto } from '@/entities/flow/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useFlows() {
  const realm = useActiveRealm()

  return useQuery<UnifiedFlowDto[]>({
    queryKey: ['flows', realm],
    queryFn: async () => {
      // The backend now returns snake_case keys (built_in, is_draft)
      // which exactly matches our UnifiedFlowDto interface.
      // No mapping is needed anymore.
      return await apiClient.get<UnifiedFlowDto[]>(`/api/realms/${realm}/flows`)
    },
    enabled: !!realm, // Prevents the query from running until realm is ready
  })
}
