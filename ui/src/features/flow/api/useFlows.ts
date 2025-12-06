import { useQuery } from '@tanstack/react-query'

import type { Flow } from '@/entities/flow/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useFlows() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['flows', realm],
    queryFn: async () => {
      return apiClient.get<Flow[]>(`/api/realms/${realm}/flows`)
    },
  })
}
